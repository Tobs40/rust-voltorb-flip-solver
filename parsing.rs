use std::path::Path;
use std::io;
use std::fs::File;
use std::io::BufRead;

pub fn string_to_level_and_constraints(
    s: &str,
) -> (usize, [usize;5], [usize;5], [usize;5], [usize;5])
{
    let mut chars: Vec<char> = s.chars().collect();
    chars.retain(|x| *x != '-');
    chars.retain(|x| *x != ' ');

    let mut a = [[127; 5]; 5];
    for i in 0..5
    {
        for j in 0..5
        {
            a[i][j] = match chars.get(i * 5 + j) {
                Some(c) => match c.to_digit(10) {
                    Some(d) => d as usize,
                    None => panic!("Failed converting char {} to integer", c),
                },
                None => panic!("Failed getting char from vector of chars"),
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
            match a[i][j]
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

    (level, sr, sc, br, bc)
}

pub fn file_to_puzzles(
    filename: &str
) -> Vec<(usize, [usize;5], [usize;5], [usize;5], [usize;5], usize)>
{
    let mut puzzles = Vec::new();

    let lines = read_lines(filename).
        expect("Failed to open file, maybe it's not there?");
    for line in lines {
        let s = line.expect("Can't read file");
        {
            let (level, sr, sc, br, bc) = string_to_level_and_constraints(&s);
            puzzles.push((puzzles.len()+1, sr, sc, br, bc, level));
        }
    }

    puzzles
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}