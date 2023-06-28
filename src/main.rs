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
struct CacheDesc {
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
    entered: u64,
}

struct CacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl CacheDesc {
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
    if line.is_empty() {
        format!("{} | -", n)
    }
    else {
        format!("{} |{}", n, line.iter().map(|x| format!(" {:x} ({}) |", x.tag, x.entered)).collect::<String>())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let (cache, addrs) = read(&args[1])?;

    let (result, stats) = simulate(&cache, &addrs);

    for (i, line) in result.iter().enumerate() {
        println!("{}", format_cache_line(line, (i as u64 / cache.n_sets()).try_into().unwrap()));
    }

    println!("Hits: {1}/{0}. Misses: {2}/{0}. Evictions: {3}/{0}", addrs.len(), stats.hits, stats.misses, stats.evictions);

    Ok(())
}

fn read(path: &str) -> Result<(CacheDesc, Vec<u64>), Box<dyn Error>> {
    let content = fs::read_to_string(path).unwrap();
    let mut lines = content.lines();
    
    let mut int_parameters = lines.by_ref()
        .take(4)
        .map(|x| x.parse::<u64>());

    // int_parameters.next() is an Option<Result<u64, IntParseError>>.
    // OkOr maps this to a Result<Result<...>, FTSError>
    // Because of the Result<Result<...>> we need to interrogate twice.
    let addr_size = int_parameters.next().ok_or(FileTooShortError)??;
    let block_size: u64 = int_parameters.next().ok_or(FileTooShortError)??;
    let n_blocks: u64 = int_parameters.next().ok_or(FileTooShortError)??;
    let assoc = int_parameters.next().ok_or(FileTooShortError)??;

    // TODO: Better error handling. This currently maps any unknown strategy to None.
    let strat = lines.next().ok_or(FileTooShortError)?.parse().ok();

    let addrs: Vec<u64> = lines
        .filter_map(|x| u64::from_str_radix(x, 16).ok())
        .collect();

    Ok((
        CacheDesc {
            addr_size,
            block_size,
            n_blocks,
            assoc,
            strat,
        },
        addrs,
    ))
}

fn simulate(cache: &CacheDesc, addrs: &Vec<u64>) -> (Vec<Vec<CacheEntry>>, CacheStats) {
    // result is a vector of cache lines. Each cache line is represented by a vector
    // that is pushed to after every step since we don't only want to know the final state
    // but also the state at each step.
    let mut result: Vec<Vec<CacheEntry>> = vec![
        vec![];
        cache.n_blocks.try_into().expect(
            "Block count too large for 32 bit machine."
        )
    ];

    let mut stats = CacheStats {
        hits: 0,
        misses: 0,
        evictions: 0
    };

    // Build masks to split address into parts.
    // Example:
    // Block Count = 16, Block Size = 16, Associativity = 4, (=> 4 Sets) 
    // addr = 100110011001
    //        ttttttiioooo
    // ( o = offset, i = set index, t = tag )
    // The masks will have a one bit in the corresponding places above.
    
    let mut idx_mask: u64 = 0;
    for j in cache.block_size..(cache.block_size + cache.idx_bits()) {
        idx_mask |= 1 << j;
    }

    let mut tag_mask: u64 = 0;
    for j in (cache.addr_size - cache.tag_bits())..cache.addr_size {
        tag_mask |= 1 << j;
    }
    
    // Iterate by index since we need to store at which iteration an access happened.
    for i in 0..addrs.len() {
        // The tag is the leftmost part of the address and needs to be shifted by the length of the
        // tail.
        let tag = (addrs[i] & tag_mask) >> (cache.block_size + cache.idx_bits());

        // The set index is to the left of the block size.
        let set_idx = (addrs[i] & idx_mask) >> cache.block_size;

        let set = &mut result[((set_idx * cache.assoc) as usize)..(((set_idx + 1) * cache.assoc)) as usize];

        // Hit! Entry in the set with matching tag was found.
        if let Some(hit) = set.iter_mut().filter_map(|x| x.last_mut()).filter(|entry| entry.tag == tag).next() {
            hit.count_used += 1;
            hit.last_used = i as u64;

            stats.hits += 1;
            continue;
        }

        stats.misses += 1;

        let new_entry = CacheEntry {
            tag,
            count_used: 1,
            last_used: i as u64,
            entered: i as u64,
        };

        match cache.strat {
            Some(Strategy::LRU) => {
                if let Some(cache_line) = set.iter_mut().filter(|x| x.is_empty()).next() {
                    // Empty = Free line found.
                    cache_line.push(new_entry);
                }
                else {
                    // No empty line found. Evict the entry where last_used is minimal.
                    // Eviction happens by appending since the last elements of the line vectors
                    // are considered to be the current state of the cache.
                    let cache_line = set.iter_mut().min_by_key(|x| x.last().unwrap().last_used);
                    cache_line.expect("Set must contain at least an empty or a full line.").push(new_entry);
                    stats.evictions += 1;
                }

            },
            Some(Strategy::LFU) => {
                if let Some(cache_line) = set.iter_mut().filter(|x| x.is_empty()).next() {
                    cache_line.push(new_entry);
                }
                else {
                    // No empty line found. Evict the entry where count_used is minimal.
                    // Eviction happens by appending since the last elements of the line vectors
                    // are considered to be the current state of the cache.
                    let cache_line = set.iter_mut().min_by_key(|x| x.last().unwrap().count_used);
                    cache_line.expect("Set must contain at least an empty or a full line.").push(new_entry);
                    stats.evictions += 1;
                }
            },
            // No eviction strategy should usually only be used with a direct (associativity = 1)
            // cache. We just assume that is the case and evict or write the first (and usually only) entry
            // in a block.
            None => {
                if !set[0].is_empty() {
                    stats.evictions += 1;
                }
                set[0].push(new_entry); 
            }
        }
    }

    (result, stats)
}
