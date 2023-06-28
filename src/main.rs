use std::str::FromStr;
use std::error::Error;
use std::{env, fs, fmt};

#[derive(Clone, Copy, Debug)]
enum Strategy {
    LRU,
    LFU,
}

impl FromStr for Strategy {
    type Err = ParseStrategyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LFU" => Ok(Strategy::LFU),
            "LRU" => Ok(Strategy::LRU),
            _ => Err(ParseStrategyError)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParseStrategyError;
impl fmt::Display for ParseStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid strategy")
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FileTooShortError;
impl fmt::Display for FileTooShortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Missing parameters.")
    }
}

impl Error for FileTooShortError {}

#[derive(Clone, Debug)]
struct Cache {
    addr_size: u64,
    block_size: u64,
    n_blocks: u64,
    assoc: u64,
    strat: Option<Strategy>,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    tag: u64,
    last_used: u64,
    count_used: u64,
}

impl Cache {
    fn tag_bits(&self) -> u64 {
        self.addr_size - self.block_size - self.n_blocks / self.assoc
    }

    fn n_sets(&self) -> u64 {
        self.n_blocks / self.assoc
    }

    fn idx_bits(&self) -> u64 {
        // log2(n_sets) = 
        (u64::BITS - self.n_sets().leading_zeros() - 1).into()
    }
}

fn format_cache_line(line: &[CacheEntry], n: u64) -> String {
    format!("{} | {:?}", n, line.last().map(|x| x.tag))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let (cache, addrs) = read(&args[1])?;

    let result = simulate(&cache, &addrs);

    for (i, line) in result.iter().enumerate() {
        println!("{}", format_cache_line(line, (i as u64 / cache.n_sets()).try_into().unwrap()));
    }

    Ok(())
}

fn read(path: &str) -> Result<(Cache, Vec<u64>), Box<dyn Error>> {
    let content = fs::read_to_string(path).unwrap();
    let mut lines = content.lines();
    
    let mut int_parameters = lines.by_ref()
        .take(4)
        .map(|x| x.parse::<u64>());

    let addr_size = int_parameters.next().ok_or(FileTooShortError)??;
    let block_size: u64 = int_parameters.next().ok_or(FileTooShortError)??;
    let n_blocks: u64 = int_parameters.next().ok_or(FileTooShortError)??;
    let assoc = int_parameters.next().ok_or(FileTooShortError)??;
    let strat = lines.next().ok_or(FileTooShortError)?.parse().ok();

    let addrs: Vec<u64> = lines
        .filter_map(|x| u64::from_str_radix(x, 16).ok())
        .collect();

    Ok((
        Cache {
            addr_size,
            block_size,
            n_blocks,
            assoc,
            strat,
        },
        addrs,
    ))
}

fn simulate(cache: &Cache, addrs: &Vec<u64>) -> Vec<Vec<CacheEntry>> {
    // result is a vector of cache lines. Each cache line is represented by a vector
    // that is pushed to after every step since we don't only want to know the final state
    // but also the state at each step.
    let mut result: Vec<Vec<CacheEntry>> = vec![
        vec![];
        cache.n_blocks.try_into().expect(
            "Block count too large for 32 bit machine."
        )
    ];

    // Iterate by index since we need to store at which iteration an access happened.
    'outer: for i in 0..addrs.len() {
        // Example:
        // Block Count = 16, Block Size = 16, Associativity = 4, (=> 4 Sets) 
        // addr = 100110011001
        //        ttttttiioooo
        // ( o = offset, i = set index, t = tag )
        let mut idx_mask: u64 = 0;
        for j in cache.block_size..cache.idx_bits() {
            idx_mask |= 1 << j;
        }

        let mut tag_mask: u64 = 0;
        for j in (64 - cache.tag_bits())..64 {
            tag_mask |= 1 << j;
        }

        let set_idx = (addrs[i] & idx_mask) >> cache.block_size;
        let mut lru: Option<u64> = None;
        let mut lfu: Option<u64> = None;
        // Either find empty block in set or least recently and least frequently used block.
        for j in (set_idx * cache.block_size)..((set_idx + 1) * cache.block_size) {
            match result[j as usize].last() {
                Some(entry) => {
                    match lru {
                        Some(lru_idx) => {
                            if result[lru_idx as usize].last().unwrap().count_used
                                < entry.count_used
                            {
                                lru = Some(j);
                            }
                        }
                        None => lru = Some(j),
                    }

                    match lfu {
                        Some(lfu_idx) => {
                            if result[lfu_idx as usize].last().unwrap().count_used
                                < entry.count_used
                            {
                                lfu = Some(j);
                            }
                        }
                        None => lfu = Some(j),
                    }
                }
                None => {
                    result[j as usize].push(CacheEntry {
                        tag: addrs[i] & tag_mask,
                        count_used: 1,
                        last_used: i as u64,
                    });
                    continue 'outer;
                }
            }

            let idx = match cache.strat {
                Some(Strategy::LFU) => lfu.unwrap() as usize,
                Some(Strategy::LRU) => lru.unwrap() as usize,
                None => set_idx as usize,
            };

            result[idx].push(CacheEntry {
                tag: addrs[i] & tag_mask,
                count_used: 1,
                last_used: i as u64,
            });
        }
    }

    result
}
