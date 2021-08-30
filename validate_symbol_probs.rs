use crate::parsing::big_database_209885;
use crate::packed::get_from_packed_state;
use crate::csp_constraints::find_possible_boards;
use crate::possible_boards::accumulate_symbol_weights;
use rayon::prelude::IntoParallelIterator;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::ParallelIterator;

// Predict the symbol probs from the root positions (no square assigned)
// sum up the probabilities for each symbol for the prediction and the database (one 100% rest 0%)
// if the algorithm is correct, they should (approximately) match

pub fn validate()
{
    for target_level in 1..=8
    {
        let puzzles: Vec<_> = big_database_209885()
            .into_iter()
            .filter(|&(.., level, _)| { level == target_level })
            .collect();

        let len_puzzles = puzzles.len();

        // sum for each symbol according to database
        let mut sum_database = [0.0; 4];
        for &(.., state) in &puzzles
        {
            for r in 0..5
            {
                for c in 0..5
                {
                    let symbol = get_from_packed_state(state, r, c);
                    sum_database[symbol] += 1.0;
                }
            }
        }

        // just creating a bunch of 3D f64 arrays 'symbol weight arrays', don't think about it in detail
        let sps: Vec<_> = puzzles.into_par_iter().map(|(_, sr, sc, br, bc, level, _)| {
            let (possible_boards, counts, weights) = find_possible_boards(0, &sr, &sc, &br, &bc, level);
            let mut indices = vec![0; weights.len() + 1];
            let mut index = 0;
            indices[0] = 0;
            for j in 0..counts.len()
            {
                index += counts[j];
                indices[j + 1] = index;
            }
            accumulate_symbol_weights(0, &possible_boards, &indices, 0, &weights)
        }).collect();

        // now I'm iterating over that vector. Couldn't / Shouldn't I write this whole snippet
        // using iterators? But I don't know how to do the part below in a nice way
        let mut sum_prediction = [0.0; 4];
        for symbol_probs in sps
        {
            for r in 0..5
            {
                for c in 0..5
                {
                    for symbol in 0..=3
                    {
                        sum_prediction[symbol] += symbol_probs[r][c][symbol];
                    }
                }
            }
        }

        // Also is there a nice iterator way to do this?
        let mut difference = [0.0; 4];
        for i in 0..4
        {
            difference[i] = (sum_prediction[i] - sum_database[i]) / len_puzzles as f64;
        }

        println!("{}", "-".repeat(200));
        println!("puzzles: {}", len_puzzles);
        println!("sum_database: {:?}", sum_database);
        println!("sum_prediction: {:?}", sum_prediction);
        println!("sum_difference: {:?}", difference);
    }
}