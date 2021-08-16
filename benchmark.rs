use std::time::Instant;
use crate::search::{compute_win_chance_exact, SearchResult};
use crate::parsing::file_to_puzzles;
use dashmap::DashMap;
use crossbeam_channel::{unbounded, RecvError};
use crate::gui::ReportMessage;

// benchmarks the algorithm on the 348 hardest puzzles using one core
// might take several minutes
pub fn benchmark()
{
    println!("Benchmarking! Might take several minutes...");

    // One thread only
    rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();

    // Loading puzzles
    let puzzles = file_to_puzzles("examples.txt");

    // remove puzzles for faster benchmarking?

    // preparing cache
    let cache_chances = DashMap::with_capacity(47_731_194);

    // Benchmarking
    let mut total_time = 0.0;

    for (nr, sr, sc, br, bc, level) in puzzles
    {
        let (control_sender, control_receiver) = unbounded();
        let (result_reporting_sender, result_reporting_receiver) = unbounded();

        cache_chances.clear();

        let search_result = compute_win_chance_exact(
            0, &sr, &sc, &br, &bc, level,
            &cache_chances,
            &control_receiver,
            &result_reporting_sender,
            &control_sender);

        if let SearchResult::SuccessfulSearchWithInfo(_win_prob, time, _nodes) = search_result
        {
            println!("Puzzle {}: {}",
                     nr, time);
            total_time += time;
        }
    }

    println!("--------------------------------------------------------------------------------");
    println!("Needed {} seconds", total_time);
}