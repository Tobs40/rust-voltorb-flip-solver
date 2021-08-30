use crate::csp_valid::{count_valid_smart, count_valid_dumb};
use crate::math::{count_special, count_symbols};
use rayon::prelude::*;
use std::sync::mpsc::sync_channel;


// returns ([bombs, ones, twos, threes], in_total, per_line) for the given level and index
pub fn get_constraint(
    level: usize,
    index: usize,
) -> ([usize;4], usize, usize)
{
    match level
    {
        1 =>
            {
                match index
                {
                    0 => ([6, 15, 3, 1], 2, 2),
                    1 => ([6, 16, 0, 3], 1, 1),
                    2 => ([6, 14, 5, 0], 3, 2),
                    3 => ([6, 15, 2, 2], 2, 2),
                    4 => ([6, 14, 4, 1], 3, 2),
                    5 => ([6, 15, 3, 1], 2, 2),
                    6 => ([6, 16, 0, 3], 1, 1),
                    7 => ([6, 14, 5, 0], 3, 2),
                    8 => ([6, 15, 2, 2], 2, 2),
                    9 => ([6, 14, 4, 1], 3, 2),
                    _ => panic!("Wrong index"),
                }
            }
        2 =>
            {
                match index
                {
                    0 => ([7, 14, 1, 3], 2, 1),
                    1 => ([7, 12, 6, 0], 3, 2),
                    2 => ([7, 13, 3, 2], 2, 1),
                    3 => ([7, 14, 0, 4], 2, 1),
                    4 => ([7, 12, 5, 1], 3, 2),
                    5 => ([7, 14, 1, 3], 1, 1),
                    6 => ([7, 12, 6, 0], 2, 2),
                    7 => ([7, 13, 3, 2], 1, 1),
                    8 => ([7, 14, 0, 4], 1, 1),
                    9 => ([7, 12, 5, 1], 2, 2),
                    _ => panic!("Wrong index"),
                }
            }
        3 =>
            {
                match index
                {
                    0 => ([8, 12, 2, 3], 2, 1),
                    1 => ([8, 10, 7, 0], 3, 2),
                    2 => ([8, 11, 4, 2], 3, 2),
                    3 => ([8, 12, 1, 4], 2, 1),
                    4 => ([8, 10, 6, 1], 2, 3),
                    5 => ([8, 12, 2, 3], 1, 1),
                    6 => ([8, 10, 7, 0], 2, 2),
                    7 => ([8, 11, 4, 2], 2, 2),
                    8 => ([8, 12, 1, 4], 1, 1),
                    9 => ([8, 10, 6, 1], 2, 2),
                    _ => panic!("Wrong index"),
                }
            }
        4 =>
            {
                match index
                {
                    0 => ([8, 11, 3, 3], 2, 3),
                    1 => ([8, 12, 0, 5], 2, 1),
                    2 => ([10, 7, 8, 0], 4, 3),
                    3 => ([10, 8, 5, 2], 3, 2),
                    4 => ([10, 9, 2, 4], 3, 2),
                    5 => ([8, 11, 3, 3], 2, 2),
                    6 => ([8, 12, 0, 5], 1, 1),
                    7 => ([10, 7, 8, 0], 3, 3),
                    8 => ([10, 8, 5, 2], 2, 2),
                    9 => ([10, 9, 2, 4], 2, 2),
                    _ => panic!("Wrong index"),
                }
            }
        5 =>
            {
                match index
                {
                    0 => ([10, 7, 7, 1], 4, 3),
                    1 => ([10, 8, 4, 3], 3, 2),
                    2 => ([10, 9, 1, 5], 3, 2),
                    3 => ([10, 6, 9, 0], 4, 3),
                    4 => ([10, 7, 6, 2], 4, 3),
                    5 => ([10, 7, 7, 1], 3, 3),
                    6 => ([10, 8, 4, 3], 2, 2),
                    7 => ([10, 9, 1, 5], 2, 2),
                    8 => ([10, 6, 9, 0], 3, 3),
                    9 => ([10, 7, 6, 2], 3, 3),
                    _ => panic!("Wrong index"),
                }
            }
        6 =>
            {
                match index
                {
                    0 => ([10, 8, 3, 4], 3, 2),
                    1 => ([10, 9, 0, 6], 3, 2),
                    2 => ([10, 6, 8, 1], 4, 3),
                    3 => ([10, 7, 5, 3], 4, 3),
                    4 => ([10, 8, 2, 5], 3, 2),
                    5 => ([10, 8, 3, 4], 2, 2),
                    6 => ([10, 9, 0, 6], 2, 2),
                    7 => ([10, 6, 8, 1], 3, 3),
                    8 => ([10, 7, 5, 3], 3, 3),
                    9 => ([10, 8, 2, 5], 2, 2),
                    _ => panic!("Wrong index"),
                }
            }
        7 =>
            {
                match index
                {
                    0 => ([10, 6, 7, 2], 4, 3),
                    1 => ([10, 7, 4, 4], 4, 3),
                    2 => ([13, 5, 1, 6], 3, 2),
                    3 => ([13, 2, 9, 1], 5, 4),
                    4 => ([10, 6, 6, 3], 4, 3),
                    5 => ([10, 6, 7, 2], 3, 3),
                    6 => ([10, 7, 4, 4], 3, 3),
                    7 => ([13, 5, 1, 6], 2, 2),
                    8 => ([13, 2, 9, 1], 4, 4),
                    9 => ([10, 6, 6, 3], 3, 3),
                    _ => panic!("Wrong index"),
                }
            }
        8 =>
            {
                match index
                {
                    0 => ([10, 8, 0, 7], 3, 2),
                    1 => ([10, 5, 8, 2], 5, 4),
                    2 => ([10, 6, 5, 4], 4, 3),
                    3 => ([10, 7, 2, 6], 4, 3),
                    4 => ([10, 5, 7, 3], 5, 4),
                    5 => ([10, 8, 0, 7], 2, 2),
                    6 => ([10, 5, 8, 2], 4, 4),
                    7 => ([10, 6, 5, 4], 3, 3),
                    8 => ([10, 7, 2, 6], 3, 3),
                    9 => ([10, 5, 7, 3], 4, 4),
                    _ => panic!("Wrong index"),
                }
            }
        _ => panic!("Wrong level"),
    }
}


// returns an array to look up weights for a specific level and index
// there's no level 0 obviously
// since it's a pretty big computation
pub fn get_weights_array() -> [[f64; 10]; 9]
{
    // how many states there are for each constraint as ordered by the get_constraint function
    let count: [[usize; 10]; 9] = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [1732660000, 81056200, 1407876600, 2598990000, 7039383000, 1732660000, 81056200, 1407876600, 2598990000, 7039383000],
        [3245678400, 5722702400, 17146024000, 811419600, 34336214400, 2684683200, 4495352000, 14348488000, 671170800, 26972112000],
        [34839212000, 12979316000, 145577634000, 17419606000, 81740052800, 32177972000, 11677150400, 128566014000, 16088986000, 81740052800],
        [171421352000, 3483921200, 18355191900, 335965442400, 204113718000, 171421352000, 3217797200, 17976411900, 331634822400, 199722318000],
        [146841535200, 559942404000, 81645487200, 13300405700, 513945373200, 143811295200, 552724704000, 79888927200, 13159573700, 503339533200],
        [559942404000, 13607581200, 119703651300, 1027890746400, 335965442400, 552724704000, 13314821200, 118436163300, 1006679066400, 331634822400],
        [478814605200, 1284863433000, 25901458800, 3265542000, 1117234078800, 473744653200, 1258348833000, 25901458800, 3265542000, 1105404190800],
        [15998354400, 400990788900, 1675851118200, 513945373200, 1069308770400, 15792134400, 394129278900, 1658106286200, 503339533200, 1051011410400]];


    let mut a = [[0.0; 10]; 9];

    for level in 1..9
    {
        for index in 0..10
        {
            let w = 1.0 / (count[level][index] as f64);
            a[level][index] = w;
        }
    }

    a
}

pub fn print_cs_array()
{
    let mut a = [[0; 10]; 9];

    let mut v = vec![];
    for level in 1..9
    {
        for index in 0..10
        {
            let t = (level,index);
            v.push(t);
        }
    }

    let results: Vec<_> = v.into_par_iter().map(|(level,index)|
        {
            let (nr_symbols, in_total, per_line) = get_constraint(level, index);
            let c = count_valid_smart(&nr_symbols, in_total, per_line);
            //println!("{}.{}, {:?} {} {}: {}", level, index, nr_symbols, in_total, per_line, c);
            (level, index, nr_symbols, in_total, per_line, c)
        }).collect();

    println!("{}", "-".repeat(100));

    for (level, index, nr_symbols, in_total, per_line, c) in results
    {
        println!("{}.{}, {:?} {} {}: {}", level, index, nr_symbols, in_total, per_line, c);
        a[level][index] = c;
    }

    println!("{}", "-".repeat(100));

    println!("{:?}", a);
}


pub fn get_weight_of_state(state: &[[usize;5];5], level: usize, cache: &[[f64; 10]; 9]) -> f64
{
    let mut weight = 0.0;
    for index in 0..10
    {
        let w = cache[level][index];
        let (nr_symbols, in_total, per_line) = get_constraint(level, index);
        if state_fits_cons(&state, &(nr_symbols, in_total, per_line))
        {
            weight += w;
        }
    }

    return weight;
}

fn state_fits_cons(state: &[[usize;5];5], cons: &([usize;4], usize, usize)) -> bool
{
    let (nr_symbols_cons, in_total, per_line) = *cons;
    let (count_total, count_line) = count_special(&state);
    let nr_symbols = count_symbols(&state);

    // same number of symbols
    for symbol in 0..4
    {
        if nr_symbols[symbol] != nr_symbols_cons[symbol]
        {
            return false;
        }
    }

    // doesn't violate any constraints for special squares
    if count_total > in_total || count_line > per_line
    {
        return false;
    }

    return true;
}