use std::{env, error::Error};

mod sim;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(Box::from(sim::InvalidArgumentsError));
    }

    let (cache, addrs) = sim::read(&args[1])?;

    let (result, stats) = sim::simulate(&cache, &addrs);

    for (i, line) in result.iter().enumerate() {
        println!("{}", sim::format_cache_line(line, (i as u64 / cache.n_sets()).try_into().unwrap()));
    }

    println!("Hits: {1}/{0}. Misses: {2}/{0}. Evictions: {3}/{0}", addrs.len(), stats.hits(), stats.misses(), stats.evictions());

    Ok(())
}