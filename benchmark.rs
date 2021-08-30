use std::time::{Instant, Duration};
use crate::search::{compute_win_chance_exact, SearchResult};
use dashmap::DashMap;
use crossbeam_channel::{unbounded, RecvError};
use crate::gui::{ReportMessage, SearchMode};
use std::thread::sleep;
use crate::parsing::examples_357;

// benchmarks the algorithm on the 348 hardest puzzles using one core
// might take several minutes
pub fn benchmark()
{
    println!("Benchmarking! Might take minutes or even hours...");

    // One thread only
    let threads = 12;

    // Loading puzzles
    let puzzles = examples_357();

    // Search mode
    let mode = SearchMode::WinEight;

    // preparing cache
    let cache = DashMap::with_capacity(111_744_155);

    // Benchmarking
    let mut total_time = 0.0;

    for (nr, sr, sc, br, bc, level, _state) in puzzles
    {
        let (control_sender, control_receiver) = unbounded();
        let (result_reporting_sender, result_reporting_receiver) = unbounded();

        cache.clear();

        let search_result = compute_win_chance_exact(
            0, &sr, &sc, &br, &bc, level,
            &cache,
            &control_receiver,
            &result_reporting_sender,
            &control_sender, mode, threads);

        if let SearchResult::SuccessfulSearchWithInfo(prob, time, nodes) = search_result
        {
            //println!("Puzzle {}: {} in {}s with {} nodes", nr, prob, time, nodes);
            println!("{}", prob);
            total_time += time;
        }
    }

    //println!("--------------------------------------------------------------------------------");
    //println!("Needed {} seconds", total_time);

    loop {
        sleep(Duration::from_secs(3600));
    }
}