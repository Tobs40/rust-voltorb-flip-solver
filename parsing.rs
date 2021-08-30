use std::path::Path;
use std::io;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::packed::array_to_u64;

pub fn string_to_level_and_constraints(
    s: &str,
) -> (usize, [usize;5], [usize;5], [usize;5], [usize;5], usize, u64)
{
    let mut chars: Vec<char> = s.chars().collect();
    chars.retain(|x| *x != '-');
    chars.retain(|x| *x != ' ');
    chars.retain(|x| *x != '\r');

    let mut state = [[127; 5]; 5];
    for i in 0..5
    {
        for j in 0..5
        {
            state[i][j] = match chars.get(i * 5 + j) {
                Some(c) => match c.to_digit(10) {
                    Some(d) => d as usize,
                    None => panic!("Failed converting char {} to integer", c),
                },
                None => panic!("Failed getting char {} from vector of chars {:?}", i * 5 + j, chars),
            };
        }
    }

    let level = match chars.get(chars.len()-1)
    {
        Some(c) => match c.to_digit(10)
        {
            Some(d) => d as usize,
            None => panic!("Failed converting char {} to integer", c),
        },
        None => panic!("Failed getting char from vector of chars"),
    };

    let mut sr = [0; 5];
    let mut sc = [0; 5];
    let mut br = [0; 5];
    let mut bc = [0; 5];

    for i in 0..5
    {
        for j in 0..5
        {
            match state[i][j]
            {
                0 => {
                    br[i] += 1;
                    bc[j] += 1;
                }
                n => {
                    sr[i] += n;
                    sc[j] += n;
                }
            }
        }
    }

    (0, sr, sc, br, bc, level, array_to_u64(&state))
}

pub fn examples_357() -> Vec<(usize, [usize;5], [usize;5], [usize;5], [usize;5], usize, u64)>
{
    string_to_puzzles(include_str!("examples.txt"))
}

pub fn hardest_5() -> Vec<(usize, [usize;5], [usize;5], [usize;5], [usize;5], usize, u64)>
{
    string_to_puzzles(include_str!("hardest.txt"))
}

pub fn big_database_209885() -> Vec<(usize, [usize;5], [usize;5], [usize;5], [usize;5], usize, u64)>
{
    string_to_puzzles(include_str!("big_database.txt"))
}

fn string_to_puzzles(s: &str) -> Vec<(usize, [usize;5], [usize;5], [usize;5], [usize;5], usize, u64)>
{
    let mut puzzles = Vec::new();
    let lines = s.split('\n');
    for line in lines {
        let (_, sr, sc, br, bc, level, state) = string_to_level_and_constraints(line);
        puzzles.push((puzzles.len()+1, sr, sc, br, bc, level, state));
    }

    puzzles
}