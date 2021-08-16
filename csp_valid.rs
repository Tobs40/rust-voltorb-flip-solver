use crate::math::{nk, nk_list, vec_to_board, split_multiset, multinomial_coefficient, count_set_squares, contains_line, count_special_locations_bomb_bitset, is_special_location_bomb_bitset, count_special};
use std::cmp::{min, max};

// board with counts
pub struct CountingBoard
{
    board: [[usize; 5]; 5],
    sum_rows: [usize; 5],
    sum_cols: [usize; 5],
    bombs_rows: [usize; 5],
    bombs_cols: [usize; 5],
    ass_rows: [usize;5],
    ass_cols: [usize;5],
    assigned: usize,
    count: [usize; 4],
}


// count states with nr_symbols which don't have too many free multipliers
// per_line max. number of free multipliers in a row or column ("line")
// in_total max. number of free multipliers in total
pub fn count_valid_dumb(nr_symbols: &[usize; 4], in_total: usize, per_line: usize) -> usize
{
    let mut cb = CountingBoard {
        board: [[127; 5]; 5],
        sum_rows: [0;5],
        sum_cols: [0;5],
        bombs_rows: [0;5],
        bombs_cols: [0;5],
        ass_rows: [0;5],
        ass_cols: [0;5],
        assigned: 0,
        count: [0; 4]
    };

    let mut sols = 0;
    let mut tries = 0;
    sh_valid(&mut cb, &nr_symbols, per_line, in_total, 0, 0, &mut sols, &mut tries);

    return sols;
}


fn sh_valid(
    cb: &mut CountingBoard,
    nr_symbols: &[usize; 4],
    per_line: usize,
    in_total: usize,
    x: usize,
    y: usize,
    sols: &mut usize,
    tries: &mut usize,
) -> ()
{
    // check number of symbols
    for symbol in 0..4
    {
        if cb.count[symbol] > nr_symbols[symbol]
        {
            return;
        }
    }

    if cb.assigned == 25
    {
        *tries += 1;
        if (*tries)%10_000_000 == 0
        {
            println!("{}, {}M", *sols, (*tries)/1_000_000);
        }

        if fits_free_multiplier_constraint(cb, in_total, per_line)
        {
            *sols += 1;
        }
    }
    else {
        for symbol in 0..4
        {
            cb.board[x][y] = symbol;
            if symbol == 0
            {
                cb.bombs_rows[x] += 1;
                cb.bombs_cols[y] += 1;
            } else {
                cb.sum_rows[x] += symbol;
                cb.sum_cols[y] += symbol;
            }
            cb.ass_rows[x] += 1;
            cb.ass_cols[y] += 1;
            cb.assigned += 1;
            cb.count[symbol] += 1;

            sh_valid(
                cb,
                &nr_symbols,
                per_line,
                in_total,
                (x * 5 + y + 1) / 5,
                (x * 5 + y + 1) % 5,
                sols,
                tries
            );

            cb.board[x][y] = 127;
            if symbol == 0
            {
                cb.bombs_rows[x] -= 1;
                cb.bombs_cols[y] -= 1;
            } else {
                cb.sum_rows[x] -= symbol;
                cb.sum_cols[y] -= symbol;
            }
            cb.ass_rows[x] -= 1;
            cb.ass_cols[y] -= 1;
            cb.assigned -= 1;
            cb.count[symbol] -= 1;
        }
    }
}


// tells if a fully assigned state is legal
fn fits_free_multiplier_constraint(
    cb: &CountingBoard,
    max_fm_total: usize,
    max_fm_per_line: usize,
) -> bool
{
    let (fm_total, fm_per_line) = count_special(&cb.board);

    if fm_total > max_fm_total || fm_per_line > max_fm_per_line
    {
        return false;
    }

    true
}


// counts valid states using smart math instead of brute force
pub fn count_valid_smart(nr_symbols: &[usize; 4], in_total: usize, per_line: usize) -> usize
{
    let mut c = 0;

    // for each way to place bombs
    for v in nk_list(25, nr_symbols[0])
    {
        // board_bombs
        let bb = vec_to_board(&v);

        // count of special locations
        let csl = count_special_locations_bomb_bitset(&bb);

        // for every number of specials that could be legal
        let max_cs = min(max(per_line, in_total), csl);
        let mut ways = vec![0; max_cs+1];

        for cs in 0..max_cs+1
        {
            if nr_symbols[1] + cs < csl // not enough 1's to fill rest of special places
            {
                continue;
            }

            let mut nr_non_special = nr_symbols.clone();
            nr_non_special[0] = 0;
            nr_non_special[1] -= csl - cs;

            for special_combo in nk_list(csl, cs)
            {
                // create board
                let mut board_special = [[false; 5]; 5];
                let mut index = 0;
                for row in 0..5
                {
                    for col in 0..5
                    {
                        if is_special_location_bomb_bitset(&bb, row, col)
                        {
                            if special_combo[index] // special square
                            {
                                board_special[row][col] = true;
                            }
                            index += 1;
                        }
                    }
                }

                // with or without line?
                // Not violating any constraints?

                let lines_illegal = contains_line(&board_special, per_line + 1);
                let total_illegal = count_set_squares(&board_special) > in_total;

                if lines_illegal || total_illegal
                {
                    // illegal board
                    continue
                }

                ways[cs] += 1;
            }

            // for every amount of 2/3's to be assigned
            for [as23, a_rest] in split_multiset(&nr_non_special, cs, [false, false, true, true])
            {
                // every way to assign this amount of 2/3's to the chosen special squares
                let ways23 = nk(as23[2] + as23[3], as23[2]);
                c += multinomial_coefficient(&a_rest) * ways23 * ways[cs];
            }
        }
    }

    return c;
}