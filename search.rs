use crate::{csp_constraints, packed};
use crate::possible_boards::{accumulate_symbol_weights, filter_possible_boards_of_next_depth};
use float_ord::FloatOrd;
use crate::math::{transpose_in_place, count_assigned, print_board, count_special, print_board_with_cons, transpose_packed, count_assigned_packed};
use std::collections::{HashSet, HashMap};
use crate::packed::{array_to_u64, board_has_possible_2_3_for_state, get_from_packed_state, set_in_packed_state, u64_to_array};
use std::fs::File;
use std::io::{BufWriter, Write};
use crate::csp_constraints::find_possible_boards;
use std::convert::TryInto;
use crate::gui::{ControlMessage, ReportMessage};
use crossbeam_channel::{Sender, Receiver, unbounded};
use crate::search::SearchResult::SuccessfulSearch;
use dashmap::DashMap;
use rayon::prelude::*;
use std::time::Instant;
use std::collections::hash_map::RandomState;
use dashmap::mapref::multiple::RefMulti;
use std::thread;
use tinyvec::array_vec;

#[derive(Copy, Clone, Debug)]
pub enum SearchResult
{
    SuccessfulSearchWithInfo(f64, f64, usize),
    SuccessfulSearch(f64),
    InconsistentPuzzle, // Not even started the search
    TerminalState, // Root state is terminal state
    Aborted,
}

pub fn compute_win_chance_exact(
    org_packed_state: u64,
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5],
    level: usize,
    cache_chances: &DashMap<u64, f64>,
    from_gui: &Receiver<ControlMessage>,
    to_gui: &Sender<ReportMessage>,
    to_thread: &Sender<ControlMessage>,
) -> SearchResult
{
    let start_of_computation = Instant::now();

    // get possible boards for original state
    let (o_pb, o_count, weights) = find_possible_boards(org_packed_state, &sr, &sc, &br, &bc, level);
    log::info!("Thread: Found {} states", o_pb.len());

    // one big array for all depths and weights,
    // an list of indices marks the start of a depth-weight group,
    // including a pointer at the end
    let mut possible_boards = Vec::with_capacity(1024 * 1024);
    let mut indices = vec![0; weights.len() * 26 + 1];

    // put in all the boards
    for i in 0..o_pb.len()
    {
        possible_boards.push(o_pb[i]);
    }

    // set the indices
    let mut index = 0;

    // depth 0 starts at index 0
    indices[0] = 0;
    // add the start indices of each weight group after that
    // including the first empty index
    for j in 0..o_count.len()
    {
        index += o_count[j];
        indices[j + 1] = index;
    }

    let acc = accumulate_symbol_weights(0, &possible_boards, &indices,
                                        0, &weights);

    if o_pb.is_empty()
    {
        return SearchResult::InconsistentPuzzle;
    } else {
        info!("Thread: Sending symbol probabilities to GUI");
        to_gui.send(ReportMessage::SquareSymbols(acc.clone()))
            .expect("Failed to send symbol prob array to GUI");
    }

    let mut squares_by_depth = Vec::with_capacity(26);
    for d in 0..26
    {
        squares_by_depth.push(Vec::with_capacity(25));
    }

    let search_result = sh_exact_root(
        org_packed_state,
        &mut squares_by_depth,
        &mut possible_boards,
        &mut indices,
        &weights,
        &sr, &sc, &br, &bc,
        cache_chances,
        &from_gui,
        &to_gui,
        &to_thread,
    );

    return if let SearchResult::SuccessfulSearch(p) = search_result
    {
        let dur = start_of_computation.elapsed();
        let nodes = cache_chances.len();

        info!("Thread: Successfully computed {} nodes in {} seconds", nodes, dur.as_secs_f64());

        SearchResult::SuccessfulSearchWithInfo(p, dur.as_secs_f64(), nodes)
    }
    else {
        search_result
    }
}

// separate root function to keep things clean and parallelize at root level
fn sh_exact_root(
    packed_state: u64,
    squares_by_depth: &mut Vec<Vec<(usize,usize)>>,
    possible_boards: &mut Vec<u64>,
    indices: &mut Vec<usize>,
    weights: &Vec<f64>,
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5],
    cache_chances: &DashMap<u64, f64>,
    from_gui: &Receiver<ControlMessage>,
    to_gui: &Sender<ReportMessage>,
    to_thread: &Sender<ControlMessage>,
) -> SearchResult
{
    let depth = 0;
    let index_start = 0;

    if is_terminal_state(packed_state, possible_boards, &indices, index_start, weights.len())
    {
        info!("Thread: Root state is terminal state, all 2/3 found");
        return SearchResult::TerminalState;
    }

    let acc = accumulate_symbol_weights(
        packed_state, possible_boards, &indices, index_start, weights
    );

    let mut jobs_per_square = [[0;5];5];

    let (send_results, rec_results) = unbounded();
    let mut jobs = Vec::with_capacity(3 * 25);
    for row in 0..5
    {
        for col in 0..5
        {
            if get_from_packed_state(packed_state,row,col) == 0
            {
                if acc[row][col][2] > 0.0 || acc[row][col][3] > 0.0
                {
                    for symbol in 1..4
                    {
                        if acc[row][col][symbol] != 0.0
                        {
                            let state = set_in_packed_state(packed_state, row, col, symbol);
                            let job = (row, col, symbol, state, squares_by_depth.clone(), possible_boards.clone(), indices.clone());
                            jobs.push(job);
                            jobs_per_square[row][col] += 1;
                        }
                    }
                }
            }
        }
    }

    let nr_jobs = jobs.len();

    jobs.into_par_iter().for_each(
        {
            |(row, col, symbol, state, mut squares_by_depth, mut possible_boards, mut indices)| {
                let search_result = sh_exact(
                    1,
                    state,
                    &mut squares_by_depth,
                    &mut possible_boards,
                    &mut indices,
                    index_start + &weights.len(),
                    &weights,
                    &sr, &sc, &br, &bc,
                    &cache_chances,
                    &from_gui,
                    &to_thread
                );

                &send_results.send((row, col, symbol, search_result))
                    .expect("Failed to send search result to root search thread");
            }
        }
    );

    let mut received = 0;
    let mut aborted = false;

    let mut highest_win_prob_so_far = 0.0;
    let mut win_chances = [[0.0;5];5]; // just for determining the max.

    while received < nr_jobs
    {
        match rec_results.recv()
        {
            Ok((row, col, symbol, search_result)) =>
                {
                    if let SearchResult::SuccessfulSearch(p) = search_result
                    {
                        win_chances[row][col] += p * acc[row][col][symbol];
                        jobs_per_square[row][col] -= 1;

                        if jobs_per_square[row][col] == 0
                        {
                            // that square is done, notify GUI about it
                            to_gui.send(ReportMessage::SquareWinProb(row, col, win_chances[row][col]))
                                .expect("Failed to send win probability for square");

                            if win_chances[row][col] > highest_win_prob_so_far
                            {
                                highest_win_prob_so_far = win_chances[row][col];
                            }
                        }
                    }
                    else if let SearchResult::Aborted = search_result {
                        aborted = true;
                    }

                    received += 1;
                }

            Err(e) => {
                panic!("Root search thread failed to receive results from search thread: {}", e);
            }
        }
    }

    return if aborted {
        SearchResult::Aborted
    } else {
        SuccessfulSearch(highest_win_prob_so_far)
    }
}

// Recursive search, basically makes up all of the runtime
fn sh_exact(
    depth: usize,
    mut packed_state: u64,
    squares_by_depth: &mut Vec<Vec<(usize,usize)>>,
    possible_boards: &mut Vec<u64>,
    indices: &mut Vec<usize>,
    index_start: usize,
    weights: &Vec<f64>,
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5],
    cache_chances: &DashMap<u64, f64>,
    from_gui: &Receiver<ControlMessage>,
    to_thread: &Sender<ControlMessage>,
) -> SearchResult
{
    if let Some(r) = cache_chances.get(&packed_state)
    {
        return SearchResult::SuccessfulSearch(*r);
    }

    // Slowing things down, make it every n steps or something again TODO
    if depth == 7
    {
        match from_gui.try_recv() // not worse than calling it every 2^17 steps at all!
        {
            Err(_) => (),
            Ok(msg) => match msg
            {
                ControlMessage::Stop => {
                    to_thread.send(ControlMessage::Stop)
                        .expect("Failed to pass Stop message on to other search threads");
                    return SearchResult::Aborted;
                },
                _ => panic!("Search thread received message other than Stop"),
            }
        }
    }

    // only create possible boards (expensive operation) after the cheaper checks
    filter_possible_boards_of_next_depth(
        packed_state,
        possible_boards,
        indices,
        index_start - weights.len(),
        weights,
    );

    let index_end = index_start + weights.len();

    if is_terminal_state(packed_state, possible_boards, &indices, index_start, index_end)
    {
        return SearchResult::SuccessfulSearch(1.0);
    }

    let pb_left = indices[index_end] - indices[index_start];

    if pb_left >= 10
    {
        match try_swap_one_row_or_col(packed_state,
                                      &sr, &sc, &br, &bc,
                                      cache_chances)
        {
            Some(r) => {
                return SearchResult::SuccessfulSearch(r);
            },
            None => (),
        }
    }

    let acc = accumulate_symbol_weights(
        packed_state, possible_boards, &indices, index_start, weights
    );

    squares_by_depth[depth].clear();
    for row in 0..5
    {
        for col in 0..5
        {
            if get_from_packed_state(packed_state,row,col) == 0
            {
                if acc[row][col][2] > 0.0 || acc[row][col][3] > 0.0
                {
                    if is_good_assignment(packed_state, &sr, &sc, &br, &bc, row, col)
                    {
                        squares_by_depth[depth].push((row, col));
                    }
                }
            }
        }
    }

    squares_by_depth[depth].sort_unstable_by_key(
        |&(row, col)| {
            FloatOrd(1024.0 * acc[row][col][0] - acc[row][col][2] - acc[row][col][3])
        }
    );

    let mut highest_win_prob_so_far = 0.0;

    for i in 0..squares_by_depth[depth].len()
    {
        let (row, col) = squares_by_depth[depth][i];

        if false
        {
            let mut opposing_square_has_already_been_searched = false;
            for j in 0..i
            {
                let (r2, c2) = squares_by_depth[depth][j];
                if same_row_or_col_group(packed_state, &sr, &sc, &br, &bc, row, col, r2, c2)
                {
                    opposing_square_has_already_been_searched = true;
                }
            }

            // skip if a square of that row-col-group has already been searched
            if opposing_square_has_already_been_searched
            {
                continue;
            }
        }

        let mut expected_value = 0.0;
        let prob_not_bomb = acc[row][col][1] + acc[row][col][2] + acc[row][col][3];
        let mut upper_bound_ev = prob_not_bomb;

        for symbol in 1..4
        {
            if acc[row][col][symbol] == 0.0
            {
                continue;
            }

            // cut if it can't be better than current best
            if expected_value + upper_bound_ev <= highest_win_prob_so_far
            {
                break;
            }

            packed_state = set_in_packed_state(packed_state, row, col, symbol);

            if let SearchResult::SuccessfulSearch(r) = sh_exact(
                depth + 1,
                packed_state,
                squares_by_depth,
                possible_boards,
                indices,
                index_start + weights.len(),
                weights,
                &sr, &sc, &br, &bc,
                cache_chances,
                &from_gui,
                &to_thread,
            ) {
                expected_value += r * acc[row][col][symbol];
                upper_bound_ev -= acc[row][col][symbol];
            } else {
                return SearchResult::Aborted;
            }
        }

        packed_state = set_in_packed_state(packed_state, row, col, 0);

        if expected_value > highest_win_prob_so_far
        {
            highest_win_prob_so_far = expected_value;
        }

        if acc[row][col][0] == 0.0
        {
            // no bomb, doesn't get any better than free information
            break;
        }
    }

    cache_chances.insert(packed_state, highest_win_prob_so_far);
    return SuccessfulSearch(highest_win_prob_so_far);
}


// Is the current square the leftmost and topmost in it's row and column group?
fn is_good_assignment(
    packed_state: u64,
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5],
    row: usize,
    col: usize,
) -> bool
{
    // rows
    for r in row+1..5
    {
        if same_row_group(packed_state, &sr, &br, r, row)
        {
            return false;
        }
    }

    // cols
    for c in col+1..5
    {
        if same_col_group(packed_state, &sc, &bc, c, col)
        {
            return false;
        }
    }

    // no equal squares before this one
    true
}


// determine whether two unassigned squares are sharing the same row
// or column group, if so, we only need to assign one of them as choosing
// either square has the same win chance
fn same_row_or_col_group(
    packed_state: u64,
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5],
    r1: usize,
    c1: usize,
    r2: usize,
    c2: usize,
) -> bool
{
    // Of course a row is in it's own row group, but that doesn't make any sense
    // Also only assigning the "opposite" (same column) square is equivalent in win chance
    if r1 != r2 && c1 == c2
    {
        if same_row_group(packed_state, &sr, &br, r1, r2)
        {
            return true;
        }
    }

    if c1 != c2 && r1 == r2
    {
        if same_col_group(packed_state, &sc, &bc, c1, c2)
        {
            return true;
        }
    }

    false
}


fn try_swap_one_row_or_col(
    mut packed_state: u64,
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5],
    cache_chances: &DashMap<u64,f64>
) -> Option<f64>
{
    // mutable copy of the state to permutate
    let mut packed_permutated_state = packed_state;

    // generate neighbours through permutation on two rows
    for r1 in 0..5
    {
        for r2 in r1+1..5
        {
            // not the same row and the same number of bombs
            if br[r1] == br[r2]
            {
                // for each way to "deassign columns" (if the n-th bit is set keep the column)
                for cols_to_keep in 0..32
                {
                    let mut da_is_feasible = true;
                    for c in 0..5
                    {
                        let keep = ((cols_to_keep >> c) & 1) == 1;
                        if keep
                        {
                            // can't keep if exactly one is assigned, the assigned/unassigned pattern wouldn't match
                            if (get_from_packed_state(packed_state, r1, c) != 0) != (get_from_packed_state(packed_state, r2, c) != 0)
                            {
                                da_is_feasible = false;
                                break;
                            }
                        }
                        else {
                            // can't deassign if there's nothing to deassign
                            if (get_from_packed_state(packed_state, r1, c) == 0) && (get_from_packed_state(packed_state, r2, c) == 0)
                            {
                                da_is_feasible = false;
                                break;
                            }
                        }
                    }

                    if da_is_feasible == false
                    {
                        continue;
                    }

                    // copy original state, but deassign those columns
                    for r in 0..5
                    {
                        for c in 0..5
                        {
                            if ((cols_to_keep >> c) & 1) == 0 && (r == r1 || r == r2)
                            {
                                packed_permutated_state = set_in_packed_state(packed_permutated_state, r, c, 0);
                            } else {
                                let v = get_from_packed_state(packed_state, r, c);
                                packed_permutated_state = set_in_packed_state(packed_permutated_state, r, c, v);
                            }
                        }
                    }

                    let mut rs1 = sr[r1];
                    let mut rs2 = sr[r2];

                    for c in 0..5
                    {
                        rs1 -= get_from_packed_state(packed_permutated_state, r1, c);
                        rs2 -= get_from_packed_state(packed_permutated_state, r2, c);
                    }

                    if rs1 != rs2
                    {
                        continue
                    }

                    for c in 0..5
                    {
                        let v1 = get_from_packed_state(packed_state, r1, c);
                        let v2 = get_from_packed_state(packed_state, r2, c);

                        // deassigned? -> swap!
                        if ((cols_to_keep >> c) & 1) == 0
                        {
                            packed_permutated_state = set_in_packed_state(packed_permutated_state, r1, c, v2);
                            packed_permutated_state = set_in_packed_state(packed_permutated_state, r2, c, v1);
                        } else {  // keep
                            packed_permutated_state = set_in_packed_state(packed_permutated_state, r1, c, v1);
                            packed_permutated_state = set_in_packed_state(packed_permutated_state, r2, c, v2);
                        }
                    }

                    let key = packed_permutated_state;

                    match cache_chances.get(&key)
                    {
                        Some(r) =>
                            {
                                return Some(*r)
                            },
                        None => (),
                    }
                }
            }
        }
    }

    // same thing for columns too using transpositions
    packed_state = transpose_packed(packed_state);
    for r1 in 0..5
    {
        for r2 in r1 + 1..5
        {
            if bc[r1] == bc[r2]
            {
                for cols_to_keep in 0..32
                {
                    let mut da_is_feasible = true;
                    for c in 0..5
                    {
                        let keep = ((cols_to_keep >> c) & 1) == 1;
                        if keep
                        {
                            // can't keep if exactly one is assigned, the assigned/unassigned pattern wouldn't match
                            if (get_from_packed_state(packed_state, r1, c) != 0) != (get_from_packed_state(packed_state, r2, c) != 0)
                            {
                                da_is_feasible = false;
                                break;
                            }
                        }
                        else {
                            // can't deassign if there's nothing to deassign
                            if (get_from_packed_state(packed_state, r1, c) == 0) && (get_from_packed_state(packed_state, r2, c) == 0)
                            {
                                da_is_feasible = false;
                                break;
                            }
                        }
                    }

                    if da_is_feasible == false
                    {
                        continue;
                    }

                    for r in 0..5
                    {
                        for c in 0..5
                        {
                            if ((cols_to_keep >> c) & 1) == 0 && (r == r1 || r == r2)
                            {
                                packed_permutated_state = set_in_packed_state(packed_permutated_state, r, c, 0);
                            } else {
                                let v = get_from_packed_state(packed_state, r, c);
                                packed_permutated_state = set_in_packed_state(packed_permutated_state, r, c, v);
                            }
                        }
                    }

                    let mut rs1 = sc[r1];
                    let mut rs2 = sc[r2];

                    for c in 0..5
                    {
                        rs1 -= get_from_packed_state(packed_permutated_state, r1, c);
                        rs2 -= get_from_packed_state(packed_permutated_state, r2, c);
                    }

                    if rs1 != rs2
                    {
                        continue
                    }

                    for c in 0..5
                    {
                        let v1 = get_from_packed_state(packed_state, r1, c);
                        let v2 = get_from_packed_state(packed_state, r2, c);

                        // deassigned? -> swap!
                        if ((cols_to_keep >> c) & 1) == 0
                        {
                            packed_permutated_state = set_in_packed_state(packed_permutated_state, r1, c, v2);
                            packed_permutated_state = set_in_packed_state(packed_permutated_state, r2, c, v1);
                        } else {  // keep
                            packed_permutated_state = set_in_packed_state(packed_permutated_state, r1, c, v1);
                            packed_permutated_state = set_in_packed_state(packed_permutated_state, r2, c, v2);
                        }
                    }



                    let key = transpose_packed(packed_permutated_state);
                    // no need to transpose back :)

                    match cache_chances.get(&key)
                    {
                        Some(r) => {
                            return Some(*r)
                        },
                        None => (),
                    }
                }
            }
        }
    }

    None
}


// same row group e.g. is the (un)assigned pattern the same and bombs and remaining sum equal?
fn same_row_group(
    packed_state: u64,
    sr: &[usize; 5],
    br: &[usize; 5],
    r1: usize,
    r2: usize,
) -> bool
{
    if br[r1] == br[r2]
    {
        // remaining sum
        let mut rs1 = sr[r1];
        let mut rs2 = sr[r2];

        // determine leftover constraints
        for c in 0..5
        {
            // either both assigned or both unassigned
            // note 0 is unassigned here
            //  (done this way because there are no bombs during search)
            if (get_from_packed_state(packed_state, r2, c) != 0) != (get_from_packed_state(packed_state, r1, c) != 0)
            {
                return false;
            }

            // if it's assigned it changes the constraints
            // but since unassigned is zero...
            rs1 -= get_from_packed_state(packed_state, r1, c);
            rs2 -= get_from_packed_state(packed_state, r2, c);
        }

        if rs1 == rs2
        {
            return true;
        }
    }

    return false;
}


// same col group e.g. is the (un)assigned pattern the same and bombs and remaining sum equal?
fn same_col_group(
    packed_state: u64,
    sc: &[usize; 5],
    bc: &[usize; 5],
    c1: usize,
    c2: usize,
) -> bool
{
    if bc[c1] == bc[c2]
    {
        // remaining sum
        let mut rs1 = sc[c1];
        let mut rs2 = sc[c2];

        // determine leftover constraints
        for r in 0..5
        {
            // either both assigned or both unassigned
            // note 0 is unassigned here
            //  (done this way because there are no bombs during search)
            if (get_from_packed_state(packed_state, r, c1) != 0) != (get_from_packed_state(packed_state, r, c2) != 0)
            {
                return false;
            }

            // if it's assigned it changes the constraints
            // but since unassigned is zero...
            rs1 -= get_from_packed_state(packed_state, r, c1);
            rs2 -= get_from_packed_state(packed_state, r, c2);
        }

        if rs1 == rs2
        {
            return true;
        }
    }

    return false;
}


// terminate if there's no 2 or 3 left (game over, you win)
fn is_terminal_state(
    packed_state: u64,
    possible_boards: &Vec<u64>,
    indices: &Vec<usize>,
    index_start: usize,
    index_end: usize,
) -> bool
{
    for i in indices[index_start]..indices[index_end]
    {
        if board_has_possible_2_3_for_state(possible_boards[i], packed_state)
        {
            return false;
        }
    }

    return true;
}
