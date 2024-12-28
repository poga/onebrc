use hashbrown::HashMap;
use memchr::memchr;
use std::env::args;
use std::io::Read;

struct Record {
    count: i32,
    min: i32,
    max: i32,
    sum: i32,
}

impl Record {
    fn default() -> Record {
        Record {
            count: 0,
            min: i32::MAX,
            max: i32::MIN,
            sum: 0,
        }
    }

    fn add(&mut self, value: i32) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn avg(&self) -> i32 {
        self.sum / self.count as i32
    }
}

fn main() {
    let filename = args().nth(1).unwrap_or("measurements.txt".to_string());
    let mut data = String::new();
    std::fs::File::open(filename)
        .expect("Could not open file")
        .read_to_string(&mut data)
        .expect("Could not read file");

    let mut h = HashMap::new();
    let mut data = data.as_bytes();
    loop {
        let Some(sep) = memchr(b';', data) else {
            break;
        };
        let end = memchr(b'\n', &data[sep..]).unwrap();
        let name = &data[..sep];
        let value = &data[sep + 1..sep + end];
        h.entry(name).or_insert(Record::default()).add(parse(value));
        data = &data[sep + end + 1..];
    }

    let mut v: Vec<_> = h.iter().collect();
    v.sort_unstable_by_key(|p| p.0);
    for (name, r) in v {
        println!(
            "{}: {}/{}/{}",
            String::from_utf8_lossy(name),
            format(r.min),
            format(r.avg()),
            format(r.max)
        );
    }
}

// parse into a fixed-precision i32 signed integer
fn parse(mut s: &[u8]) -> i32 {
    let neg = if s[0] == b'-' {
        s = &s[1..];
        true
    } else {
        false
    };
    // s = abc.d
    let (a, b, c, d) = match s {
        [c, b'.', d] => (0, 0, c - b'0', d - b'0'),
        [b, c, b'.', d] => (0, b - b'0', c - b'0', d - b'0'),
        [a, b, c, b'.', d] => (a - b'0', b - b'0', c - b'0', d - b'0'),
        [c] => (0, 0, 0, c - b'0'),
        [b, c] => (0, b - b'0', c - b'0', 0),
        [a, b, c] => (a - b'0', b - b'0', c - b'0', 0),
        _ => panic!("Unknown patters {:?}", std::str::from_utf8(s).unwrap()),
    };
    let v = a as i32 * 1000 + b as i32 * 100 + c as i32 * 10 + d as i32;
    if neg {
        -v
    } else {
        v
    }
}

fn format(v: i32) -> String {
    format!("{:.1}", v as f64 / 10.0)
}
