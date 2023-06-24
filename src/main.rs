use std::io::{Error, Read, ErrorKind};
use std::{fs, env};

#[derive(Clone)]
struct Cache {
    addr_size: u64,
    block_size: u64,
    n_blocks: u64,
    assoc: u64
}

#[derive(Clone)]
struct CacheEntry {
    tag: u64,
    last_used: u64,
    count_used: u64
}

impl Cache {
    fn tag_bits(&self) -> u64 {
        self.addr_size - self.block_size - self.n_blocks / self.assoc
    }

    fn n_sets(&self) -> u64 {
        self.n_blocks / self.assoc
    }

    fn idx_bits(&self) -> u32 {
        u64::BITS - self.assoc.leading_zeros()
    }
}

fn format_cache_line(line: &Vec<CacheEntry>, n: u64) -> String {
    format!("{} | {:?}", n, line.last().map(|x| x.tag))
}
fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    println!("{}", args[1]);
    let (cache, addrs) = read(&args[1])?;
    let result = simulate(&cache, &addrs);
    for (i, line) in result.iter().enumerate() {
        println!("{}", format_cache_line(line, i.try_into().unwrap()));
    }
    println!("Hello, world!");
    println!("{}", addrs[0]);
    Ok(())
}

fn read(path: &str) -> Result<(Cache, Vec<u64>), Error> {
    let content = fs::read_to_string(path).unwrap();
    let mut lines = content.lines();
    let size = lines.next().ok_or(Error::new(ErrorKind::Other, "Input too short."))?;
    let addr_size: u64 = size.parse().unwrap();
    let block_size = lines.next().unwrap();
    let block_size: u64 = block_size.parse().unwrap();
    let n_blocks = lines.next().unwrap();
    let n_blocks: u64 = n_blocks.parse().unwrap();
    let assoc = lines.next().unwrap();
    let assoc: u64 = assoc.parse().unwrap();
    let addrs: Vec<u64> = lines.filter_map(|x| u64::from_str_radix(x, 16).ok()).collect();
    Ok((Cache {addr_size, block_size, n_blocks, assoc}, addrs))
} 

fn simulate(cache: &Cache, addrs: &Vec<u64>) -> Vec<Vec<CacheEntry>> {
    let result: Vec<Vec<CacheEntry>> = vec![vec![]; cache.n_blocks.try_into().expect("Block count too large for 32 bit machine.")];
    for i in 0..addrs.len() {
        
    }
    vec![vec![]]
}
