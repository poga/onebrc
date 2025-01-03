use hashbrown::HashMap;
use memchr::memchr;
use memchr::memrchr;
use std::collections::HashSet;
use std::env::args;
use std::io::Read;
use std::sync::Mutex;
use std::thread;
use std::thread::available_parallelism;

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

    fn merge(&mut self, other: &Self) {
        self.count += other.count;
        self.sum += other.sum;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
}

fn run(mut data: &[u8]) -> HashMap<&[u8], Record> {
    let mut h = HashMap::new();

    loop {
        let Some(sep) = memchr(b';', data) else {
            break;
        };
        let Some(end) = memchr(b'\n', &data[sep..]) else {
            println!(
                "No newline found for {:?}",
                std::str::from_utf8(data).unwrap()
            );
            break;
        };
        let name = &data[..sep];
        let value = &data[sep + 1..sep + end];
        h.entry(name).or_insert(Record::default()).add(parse(value));
        data = &data[sep + end + 1..];
    }

    h
}

fn main() {
    let filename = args().nth(1).unwrap_or("measurements.txt".to_string());
    let mut data = String::new();
    std::fs::File::open(filename)
        .expect("Could not open file")
        .read_to_string(&mut data)
        .expect("Could not read file");

    let results = Mutex::new(Vec::new());
    let cities = Mutex::new(HashSet::new());
    let data = data.as_bytes();
    let num_threads = available_parallelism().unwrap();
    thread::scope(|s| {
        let chunk_size = data.len() / num_threads;

        // make sure each chunk starts at a newline and ends at a newline
        let mut start = 0;
        loop {
            if start + chunk_size >= data.len() {
                break;
            }
            let chunk = &data[start..(start + chunk_size)];
            let end = memrchr(b'\n', chunk).unwrap() + 1;
            let data = &data[start..(start + end)];
            /*
            let head = std::str::from_utf8(&data[0..30]).unwrap();
            let tail = std::str::from_utf8(&data[(data.len() - 30)..]).unwrap();
            println!("{:?}...{:?}", head, tail);
            */
            s.spawn(|| {
                let rec = run(data);

                cities.lock().unwrap().extend(rec.keys());
                results.lock().unwrap().push(rec);
            });
            start += chunk.len() - (chunk.len() - end);
        }
    });

    let cities = cities.lock().unwrap();
    let mut sorted_cities = cities.iter().collect::<Vec<_>>();
    sorted_cities.sort();
    for name in sorted_cities {
        let r = results
            .lock()
            .unwrap()
            .iter()
            .fold(Record::default(), |acc, x| {
                if let Some(v) = x.get(*name) {
                    let mut acc = acc;
                    acc.merge(v);
                    acc
                } else {
                    acc
                }
            });
        println!(
            "{:?}: {}/{}/{}",
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
