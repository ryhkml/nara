use std::f64::consts::TAU;
use std::fs;
use std::path::Path;

const RATE: usize = 48000;

struct Noise(u64);

impl Noise {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64 * 2. - 1.
    }
}

struct Biquad {
    b: [f64; 3],
    a: [f64; 2],
    x: [f64; 2],
    y: [f64; 2],
}

impl Biquad {
    fn bandpass(freq: f64, q: f64) -> Self {
        let mut f = Self {
            b: [0.; 3],
            a: [0.; 2],
            x: [0.; 2],
            y: [0.; 2],
        };
        f.tune_bandpass(freq, q);
        f
    }

    fn tune_bandpass(&mut self, freq: f64, q: f64) {
        let w = TAU * freq / RATE as f64;
        let alpha = w.sin() / (2. * q);
        let a0 = 1. + alpha;
        self.b = [alpha / a0, 0., -alpha / a0];
        self.a = [-2. * w.cos() / a0, (1. - alpha) / a0];
    }

    fn process(&mut self, x: f64) -> f64 {
        let y = self.b[0] * x + self.b[1] * self.x[0] + self.b[2] * self.x[1]
            - self.a[0] * self.y[0]
            - self.a[1] * self.y[1];
        self.x = [x, self.x[0]];
        self.y = [y, self.y[0]];
        y
    }
}

fn midi(note: f64) -> f64 {
    440. * 2f64.powf((note - 69.) / 12.)
}

fn at(seconds: f64) -> usize {
    (seconds * RATE as f64).round() as usize
}

fn piano(buf: &mut [f64], start: f64, note: f64, velocity: f64) {
    let f0 = midi(note);
    let s0 = at(start);
    let tau0 = 1.5 * (261.63 / f0).powf(0.35);
    for n in 1..=8 {
        let n = n as f64;
        let freq = n * f0 * (1. + 0.0003 * n * n).sqrt();
        if freq > 9000. {
            break;
        }
        let amp = velocity * velocity.powf(0.4 * (n - 1.)) / n.powf(1.7);
        let tau = tau0 / (1. + 0.75 * (n - 1.));
        let len = at(tau * 7.).min(buf.len().saturating_sub(s0));
        for i in 0..len {
            let t = i as f64 / RATE as f64;
            let env = (t / 0.004).min(1.) * (-t / tau).exp();
            let wave = (TAU * freq * t).sin() + (TAU * freq * 1.0009 * t + 0.7).sin();
            buf[s0 + i] += amp * env * wave * 0.5;
        }
    }
}

fn pad(buf: &mut [f64], start: f64, length: f64, notes: &[f64], level: f64) {
    let (attack, release) = (1.1, 1.4);
    let s0 = at(start);
    let len = at(length + release).min(buf.len().saturating_sub(s0));
    for (k, &note) in notes.iter().enumerate() {
        for (d, cents) in [-6., 0., 6.].iter().enumerate() {
            let f0 = midi(note) * 2f64.powf(cents / 1200.);
            for n in 1..=5 {
                let n = n as f64;
                let amp = level / (n * n) / 3.;
                let phase = (k * 7 + d * 3) as f64 + n;
                for i in 0..len {
                    let t = i as f64 / RATE as f64;
                    let rise = 0.5 - 0.5 * (std::f64::consts::PI * (t / attack).min(1.)).cos();
                    let fall = if t > length {
                        0.5 + 0.5 * (std::f64::consts::PI * ((t - length) / release).min(1.)).cos()
                    } else {
                        1.
                    };
                    let swell = 1. + 0.08 * (TAU * 0.13 * (start + t) + k as f64).sin();
                    buf[s0 + i] += amp * rise * fall * swell * (TAU * n * f0 * t + phase).sin();
                }
            }
        }
    }
}

fn bass(buf: &mut [f64], start: f64, length: f64, note: f64, level: f64) {
    let f0 = midi(note);
    let s0 = at(start);
    let len = at(length + 0.3).min(buf.len().saturating_sub(s0));
    for i in 0..len {
        let t = i as f64 / RATE as f64;
        let env = (t / 0.03).min(1.)
            * (-t / 1.6).exp()
            * if t > length {
                1. - (t - length) / 0.3
            } else {
                1.
            };
        let wave = (TAU * f0 * t).sin()
            + 0.3 * (TAU * 2. * f0 * t).sin()
            + 0.1 * (TAU * 3. * f0 * t).sin();
        buf[s0 + i] += level * env * wave;
    }
}

fn reverb(dry: &[f64], wet: f64) -> Vec<f64> {
    let scale = RATE as f64 / 44100.;
    let mut combs: Vec<(Vec<f64>, usize, f64)> = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617]
        .iter()
        .map(|&d| (vec![0.; (d as f64 * scale) as usize], 0, 0.))
        .collect();
    let mut allpasses: Vec<(Vec<f64>, usize)> = [556, 441, 341, 225]
        .iter()
        .map(|&d| (vec![0.; (d as f64 * scale) as usize], 0))
        .collect();
    let (feedback, damp) = (0.84, 0.25);
    dry.iter()
        .map(|&x| {
            let input = x * 0.015;
            let mut out = 0.;
            for (line, pos, store) in combs.iter_mut() {
                let y = line[*pos];
                *store = y * (1. - damp) + *store * damp;
                line[*pos] = input + *store * feedback;
                *pos = (*pos + 1) % line.len();
                out += y;
            }
            for (line, pos) in allpasses.iter_mut() {
                let y = line[*pos];
                line[*pos] = out + y * 0.5;
                *pos = (*pos + 1) % line.len();
                out = y - out;
            }
            x + out * wet * 3.
        })
        .collect()
}

fn normalize(buf: &mut [f64], peak: f64) {
    let max = buf.iter().fold(0f64, |m, &x| m.max(x.abs()));
    if max > 0. {
        buf.iter_mut().for_each(|x| *x *= peak / max);
    }
}

fn write_wav(path: &Path, buf: &[f64]) {
    let data: Vec<u8> = buf
        .iter()
        .flat_map(|&x| ((x.clamp(-1., 1.) * 32767.).round() as i16).to_le_bytes())
        .collect();
    let mut out = Vec::with_capacity(44 + data.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(RATE as u32).to_le_bytes());
    out.extend_from_slice(&(RATE as u32 * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&data);
    fs::write(path, out).expect("write wav");
    println!("{} {:.2}s", path.display(), buf.len() as f64 / RATE as f64);
}

struct Chord {
    pad: [f64; 4],
    arpeggio: [f64; 4],
    root: f64,
}

const C: Chord = Chord {
    pad: [55., 60., 64., 71.],
    arpeggio: [60., 64., 67., 72.],
    root: 48.,
};
const AM: Chord = Chord {
    pad: [57., 60., 64., 67.],
    arpeggio: [57., 60., 64., 69.],
    root: 45.,
};
const F: Chord = Chord {
    pad: [53., 57., 60., 64.],
    arpeggio: [60., 65., 69., 72.],
    root: 41.,
};
const G: Chord = Chord {
    pad: [55., 59., 62., 67.],
    arpeggio: [59., 62., 67., 71.],
    root: 43.,
};
const DM: Chord = Chord {
    pad: [53., 57., 60., 62.],
    arpeggio: [62., 65., 69., 72.],
    root: 38.,
};

fn music(outro: f64, total: f64) -> Vec<f64> {
    let bars = (outro / 3.).round().max(4.) as usize;
    let bar = outro / bars as f64;
    let eighth = bar / 8.;
    let mut keys = vec![0.; at(total + 4.)];
    let mut pads = vec![0.; keys.len()];
    let mut low = vec![0.; keys.len()];

    let shape = [0usize, 1, 2, 3, 4, 3, 2, 1];
    let accents = [0.8, 0.5, 0.6, 0.55, 0.7, 0.5, 0.55, 0.45];
    for b in 0..bars {
        let chord = if b + 4 >= bars {
            [&AM, &F, &DM, &G][b + 4 - bars]
        } else {
            [&C, &AM, &F, &G][b % 4]
        };
        let start = b as f64 * bar;
        pad(&mut pads, start, bar, &chord.pad, 0.5);
        if b > 0 {
            bass(&mut low, start, bar / 2., chord.root, 0.5);
            bass(&mut low, start + bar / 2., bar / 2., chord.root, 0.35);
        }
        for (k, &step) in shape.iter().enumerate() {
            let note = if step == 4 {
                chord.arpeggio[0] + 12.
            } else {
                chord.arpeggio[step]
            };
            let swing = ((b * 8 + k) * 37 % 7) as f64 * 0.002;
            piano(
                &mut keys,
                start + k as f64 * eighth + swing,
                note,
                accents[k],
            );
        }
    }

    let ring = total - outro + 1.;
    for (k, note) in [36., 43., 48., 55., 60., 64., 67., 72.].iter().enumerate() {
        piano(
            &mut keys,
            outro + k as f64 * 0.03,
            *note,
            0.75 - k as f64 * 0.03,
        );
    }
    pad(&mut pads, outro, ring, &[55., 60., 64., 67., 74.], 0.5);
    bass(&mut low, outro, ring, 36., 0.5);

    let mixed: Vec<f64> = keys.iter().zip(&pads).map(|(k, p)| k + p * 0.45).collect();
    let mut out: Vec<f64> = reverb(&mixed, 0.28)
        .iter()
        .zip(&low)
        .map(|(m, l)| m + l * 0.5)
        .collect();
    out.truncate(at(total));
    let mut previous = 0.;
    let mut high = 0.;
    for x in out.iter_mut() {
        high = 0.995 * (high + *x - previous);
        previous = *x;
        *x = high;
    }
    let fade = at(0.3);
    for (i, x) in out.iter_mut().take(fade).enumerate() {
        *x *= i as f64 / fade as f64;
    }
    normalize(&mut out, 0.89);
    out
}

fn click() -> Vec<f64> {
    let mut buf = vec![0.; at(0.16)];
    let mut noise = Noise(0x9E3779B97F4A7C15);
    for (offset, level, freq) in [(0., 1., 3000.), (0.072, 0.45, 3600.)] {
        let mut filter = Biquad::bandpass(freq, 1.2);
        let s0 = at(offset);
        for i in 0..at(0.03) {
            let t = i as f64 / RATE as f64;
            let tick = filter.process(noise.next()) * (-t / 0.0012).exp();
            let body = (TAU * 1900. * t).sin() * (-t / 0.002).exp() * 0.6;
            buf[s0 + i] += level * (tick * 3. + body);
        }
    }
    normalize(&mut buf, 0.89);
    buf
}

fn key(seed: u64, body_freq: f64, click_freq: f64) -> Vec<f64> {
    let mut buf = vec![0.; at(0.12)];
    let mut noise = Noise(seed);
    for (offset, level) in [(0., 1.), (0.028, 0.35)] {
        let mut filter = Biquad::bandpass(click_freq, 1.);
        let s0 = at(offset);
        for i in 0..at(0.06) {
            let t = i as f64 / RATE as f64;
            let tick = filter.process(noise.next()) * (-t / 0.0025).exp();
            let body = (TAU * body_freq * t).sin() * (-t / 0.012).exp() * 0.5;
            buf[s0 + i] += level * (tick * 2. + body);
        }
    }
    normalize(&mut buf, 0.89);
    buf
}

fn bell(buf: &mut [f64], start: f64, freq: f64, level: f64, tau: f64, partial: f64) {
    let s0 = at(start);
    for i in 0..buf.len() - s0 {
        let t = i as f64 / RATE as f64;
        let attack = (t / 0.003).min(1.);
        let tone = (TAU * freq * t).sin() * (-t / tau).exp()
            + 0.3 * (TAU * freq * partial * t).sin() * (-t / (tau * 0.25)).exp()
            + 0.08 * (TAU * freq * 2. * t).sin() * (-t / (tau * 0.5)).exp();
        buf[s0 + i] += level * attack * tone;
    }
}

fn chime() -> Vec<f64> {
    let mut buf = vec![0.; at(1.6)];
    for (k, note) in [79., 84., 88.].iter().enumerate() {
        bell(
            &mut buf,
            k as f64 * 0.085,
            midi(*note),
            1. - k as f64 * 0.12,
            0.55,
            2.76,
        );
    }
    let mut out = reverb(&buf, 0.2);
    normalize(&mut out, 0.89);
    out
}

fn ding() -> Vec<f64> {
    let mut buf = vec![0.; at(1.1)];
    bell(&mut buf, 0., midi(81.), 0.8, 0.32, 4.);
    bell(&mut buf, 0.11, midi(88.), 1., 0.4, 4.);
    let mut out = reverb(&buf, 0.15);
    normalize(&mut out, 0.89);
    out
}

fn whoosh() -> Vec<f64> {
    let length = 1.1;
    let mut buf = vec![0.; at(length)];
    let mut noise = Noise(0xD1B54A32D192ED03);
    let mut filter = Biquad::bandpass(300., 0.9);
    for (i, x) in buf.iter_mut().enumerate() {
        let t = i as f64 / RATE as f64;
        let k = t / length;
        if i % 32 == 0 {
            let peak = 0.5;
            let freq = if k < peak {
                250. * (2600f64 / 250.).powf(k / peak)
            } else {
                2600. * (700f64 / 2600.).powf((k - peak) / (1. - peak))
            };
            filter.tune_bandpass(freq, 0.9);
        }
        let env = (std::f64::consts::PI * k).sin().powi(2);
        *x = filter.process(noise.next()) * env;
    }
    normalize(&mut buf, 0.89);
    buf
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() == 4,
        "usage: sound <media dir> <outro seconds> <total seconds>"
    );
    let dir = Path::new(&args[1]);
    let outro: f64 = args[2].parse().expect("outro seconds");
    let total: f64 = args[3].parse().expect("total seconds");
    assert!(
        outro > 12. && total > outro,
        "outro must sit inside the video"
    );

    write_wav(&dir.join("music.wav"), &music(outro, total));
    write_wav(&dir.join("click.wav"), &click());
    for (k, (body, tick)) in [(180., 2400.), (205., 3100.), (225., 2700.), (195., 3500.)]
        .iter()
        .enumerate()
    {
        write_wav(
            &dir.join(format!("key_{}.wav", k + 1)),
            &key(0xA5A5 + k as u64 * 7919, *body, *tick),
        );
    }
    write_wav(&dir.join("chime.wav"), &chime());
    write_wav(&dir.join("ding.wav"), &ding());
    write_wav(&dir.join("whoosh.wav"), &whoosh());
}
