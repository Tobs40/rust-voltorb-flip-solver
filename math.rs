use std::cmp::{min, max};
use crate::packed::{get_from_packed_state, set_in_packed_state};


// n choose k
// 0 for k > n
// uses multiplication formula
// no caching
// only usize accuracy
pub fn nk(n:usize, k:usize) -> usize
{
    if k > n {
        return 0;
    }

    let mut ans = 1;

    let mut n = n;
    let k = min(k, n-k);

    for j in 1..k+1
    {
        if n%j == 0
        {
            ans *= n/j;
        } else if ans%j == 0
        {
            ans = ans/j*n;
        }
        else {
            ans = (ans*n)/j;
        }
        n -= 1;
    }

    return ans;
}

// number of ways to assign length(numbers) colors
// to sum(numbers) distinguishable places such that
// color i appears exactly numbers[i] times
pub fn multinomial_coefficient(numbers: &[usize]) -> usize
{
    let mut left = 0;
    for num in numbers
    {
        left += num;
    }

    let mut r = 1;
    for &num in numbers
    {
        r *= nk(left, num);
        left -= num;
    }

    return r;
}


// vec of all ways to choose k from n
pub fn nk_list(n: usize, k: usize) -> Vec<Vec<bool>>
{
    let mut r = Vec::with_capacity(nk(n, k));
    _nk_list_helper(n, k, &mut vec![false; n], 0, 0, &mut r);
    return r;
}


fn _nk_list_helper(
    n: usize,
    k: usize,
    current: &mut Vec<bool>,
    assigned: usize,
    index: usize,
    v: &mut Vec<Vec<bool>>
)
{
    if assigned == k
    {
        let rex = current.clone();
        v.push(rex);
    }
    else if index < current.len() {
        current[index] = false;
        _nk_list_helper(n, k, current, assigned, index+1, v);
        current[index] = true;
        _nk_list_helper(n, k, current, assigned+1, index+1, v);
        current[index] = false;
    }
}


// convert vector of length 25 to 5x5 array (board)
pub fn vec_to_board(v: &Vec<bool>) -> [[bool;5];5]
{
    let mut a = [[false;5];5];
    for i in 0..25
    {
        a[i/5][i%5] = v[i];
    }
    return a;
}

// count the total number of special squares
// and the max. number of special squares in a row or column
pub fn count_special(state: &[[usize;5];5]) -> (usize, usize)
{
    let mut count_total = 0;
    let mut rows_special = [0;5];
    let mut cols_special = [0;5];

    for r in 0..5
    {
        for c in 0..5
        {
            if is_square_special(&state, r, c)
            {
                count_total += 1;
                rows_special[r] += 1;
                cols_special[c] += 1;
            }
        }
    }

    let mut count_line = 0;
    for i in 0..5
    {
        count_line = max(count_line, rows_special[i]);
        count_line = max(count_line, cols_special[i]);
    }

    return (count_total, count_line);
}

// WRONG TODO fix and recompute the counts!
fn is_square_special(state: &[[usize;5];5], row: usize, col: usize) -> bool
{
    (state[row][col] == 2 || state[row][col] == 3) && is_special_location(&state, row, col)
}


pub fn count_symbols(state: &[[usize;5];5]) -> [usize;4]
{
    let mut count = [0; 4];

    for r in 0..5
    {
        for c in 0..5
        {
            count[state[r][c]] += 1;
        }
    }

    return count;
}


pub fn count_assigned(state: &[[usize;5];5]) -> usize
{
    let mut count = 0;

    for r in 0..5
    {
        for c in 0..5
        {
            if state[r][c] != 0
            {
                count += 1;
            }
        }
    }

    return count;
}


pub fn count_assigned_packed(packed_state: u64) -> usize
{
    let mut count = 0;

    for r in 0..5
    {
        for c in 0..5
        {
            if get_from_packed_state(packed_state,r,c) != 0
            {
                count += 1;
            }
        }
    }

    return count;
}


// contains at least k true values in a row or column
pub fn contains_line(a: &[[bool;5];5], k: usize) -> bool
{
    for line in 0..5
    {
        let mut rc = 0;
        let mut cc = 0;
        for i in 0..5
        {
            if a[line][i]
            {
                rc += 1;
            }
            if a[i][line]
            {
                cc += 1;
            }
        }
        if rc >= k || cc >= k
        {
            return true;
        }
    }

    return false;
}


// count how many squares are special locations
// special locations = no bombs on it's row or no bombs on it's column
// when a special location is occupied by a 2 or 3, it's a special square
pub fn count_special_locations_bomb_bitset(a: &[[bool;5];5]) -> usize
{
    let mut count = 0;
    for row in 0..5
    {
        for col in 0..5
        {
            if is_special_location_bomb_bitset(&a, row, col)
            {
                count += 1;
            }
        }
    }

    return count;
}


// given boolean array of bombs=true, tell whether a given location is special
// (no bombs in row or no bombs in column)
pub fn is_special_location_bomb_bitset(a: &[[bool;5];5], row: usize, col: usize) -> bool
{
    let mut bombs_on_row = false;
    let mut bombs_on_col = false;
    for i in 0..5
    {
        if a[row][i]
        {
            bombs_on_row = true;
        }
        if a[i][col]
        {
            bombs_on_col = true;
        }
    }

    (!bombs_on_row) || (!bombs_on_col)
}


pub fn is_special_location(state: &[[usize;5];5], row: usize, col: usize) -> bool
{
    let mut bombs_on_row = false;
    let mut bombs_on_col = false;
    for i in 0..5
    {
        if state[row][i] == 0
        {
            bombs_on_row = true;
        }
        if state[i][col] == 0
        {
            bombs_on_col = true;
        }
    }

    (!bombs_on_row) || (!bombs_on_col)
}


// count how many squares are true
pub fn count_set_squares(a: &[[bool;5];5]) -> usize
{
    let mut count = 0;
    for r in 0..5
    {
        for c in 0..5
        {
            if a[r][c]
            {
                count += 1;
            }
        }
    }

    return count;
}


// all ways to separate certain amounts of the four symbols into two
// depending on boolean array only. False at only[symbol] means that none of
// that symbol may be taken
pub fn split_multiset(
    available_symbols: &[usize;4], // how many of each symbol there are
    size_fms: usize, // number of symbols in first multiset
    only: [bool;4],
) -> Vec<[[usize;4];2]>
{
    let mut v = Vec::new();
    split_multiset_helper(&mut [0;4], 0, 0, &only, &available_symbols, size_fms, &mut v);
    return v;
}


fn split_multiset_helper(
    current: &mut[usize;4],
    size: usize,
    index: usize,
    only: &[bool;4],
    available_symbols: &[usize;4],
    size_fms: usize,
    v: &mut Vec<[[usize;4];2]>,
)
{
    if index == 4 && size == size_fms
    {
        let mut opposite = available_symbols.clone();
        for symbol in 0..4
        {
            opposite[symbol] -= current[symbol];
        }
        let ta = [current.clone(), opposite];
        v.push(ta);
    }
    else if index < 4
    {
        for c in 0..available_symbols[index]+1
        {
            if size+c <= size_fms && (c == 0 || only[index])
            {
                current[index] = c;
                split_multiset_helper(current, size+c, index+1, &only, &available_symbols, size_fms, v);
            }
        }
    }
}

// swap rows and columns of array a: a[i][j] <-> a[j][i] for all i, j
pub fn transpose_in_place(a: &mut [[usize;5];5])
{
    for row in 0..5
    {
        for col in row+1..5
        {
            let tmp = a[row][col];
            a[row][col] = a[col][row];
            a[col][row] = tmp;
        }
    }
}


// swap rows and columns of array a: a[i][j] <-> a[j][i] for all i, j
pub fn transpose_packed(packed_state: u64) -> u64
{
    let mut transposed_packed_state = 0;
    for row in 0..5
    {
        for col in 0..5
        {
            let value = get_from_packed_state(packed_state,row,col);
            transposed_packed_state = set_in_packed_state(transposed_packed_state,col,row, value);
        }
    }

    transposed_packed_state
}


pub fn print_board(board: &[[usize; 5]; 5]) {
    for row in 0..5 {
        for col in 0..5 {
            if board[row][col] == 0
            {
                print!(". ", );
            } else {
                print!("{} ", board[row][col]);
            }
        }
        println!();
    }
}


pub fn print_board_with_cons(
    board: &[[usize; 5]; 5],
    sr: &[usize;5],
    sc: &[usize;5],
    br: &[usize;5],
    bc: &[usize;5],
)
{
    for col in 0..5
    {
        print!("{} ", sc[col]);
    }

    println!();

    for col in 0..5
    {
        print!("{} ", bc[col]);
    }

    println!();

    for _ in 0..5
    {
        print!("--");
    }

    println!();


    for row in 0..5 {
        for col in 0..8 {
            if col == 5
            {
                print!("| ");
            }
            else if col == 6
            {
                print!("{} ", br[row]);
            }
            else if col == 7
            {
                print!("{} ", sr[row]);
            }
            else {
                if board[row][col] == 0
                {
                    print!(". ", );
                }
                else {
                    print!("{} ", board[row][col]);
                }
            }
        }
        println!();
    }
}