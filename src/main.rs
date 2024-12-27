use std::collections::HashMap;
use std::env::args;
use std::io::Read;

struct Record {
    count: i32,
    min: f32,
    max: f32,
    sum: f32,
}

impl Record {
    fn default() -> Record {
        Record {
            count: 0,
            min: f32::INFINITY,
            max: f32::NEG_INFINITY,
            sum: 0.0,
        }
    }

    fn add(&mut self, value: f32) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn avg(&self) -> f32 {
        self.sum / self.count as f32
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
    for line in data.lines() {
        let (name, value) = line.split_once(';').unwrap();
        let value: f32 = value.parse().expect("Could not parse value");
        h.entry(name.to_string())
            .or_insert(Record::default())
            .add(value);
    }

    let mut v: Vec<_> = h.iter().collect();
    v.sort_unstable_by_key(|p| p.0);
    for (name, r) in v {
        println!("{name}: {:.1}/{:.1}/{:.1}", r.min, r.avg(), r.max);
    }
}
