use crate::level_constraints::{get_weight_of_state, get_weights_array};
use crate::packed::{array_to_u64, get_from_packed_state};
use crate::math::print_board;


// returns possible boards sorted by weight,
// how many there are of weight i
// what weight i is
pub fn find_possible_boards(
    org_packed_state: u64,
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5],
    level: usize,
) -> (Vec<u64>, Vec<usize>, Vec<f64>)
{
    let mut r = Vec::new();

    let mut state = [[127; 5]; 5];
    for r in 0..5
    {
        for c in 0..5
        {
            let symbol = get_from_packed_state(org_packed_state, r, c);
            if symbol != 0
            {
                state[r][c] = symbol;
            }
        }
    }

    sh_constraints(&mut state, 0, 0, &sr, &sc, &br, &bc, level, &mut r, &get_weights_array());

    let mut possible_boards = Vec::new();
    let mut counts = Vec::with_capacity(r.len());
    let mut weights = Vec::with_capacity(r.len());

    for (pbs, w) in r
    {
        counts.push(pbs.len());
        weights.push(w);
        for b in pbs
        {
            possible_boards.push(b);
        }
    }

    (possible_boards, counts, weights)
}


fn sh_constraints(
    state: &mut [[usize; 5]; 5],
    row: usize,
    col: usize,
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5],
    level: usize,
    solutions: &mut Vec<(Vec<u64>, f64)>,
    cache: &[[f64; 10]; 9],
)
{
    if !fsf(&state, &sr, &sc, &br, &bc)
    {
        return;
    }

    if row == 5
    {
        let weight = get_weight_of_state(&state, level, &cache);
        if weight > 0.0 // fits in at least one group of the level
        {
            let packed_state = array_to_u64(&state);
            let mut contains = false;
            for i in 0..solutions.len()
            {
                let (pbs,w) = &mut solutions[i];
                if weight == *w
                {
                    pbs.push(packed_state);
                    contains = true;
                    break;
                }
            }
            if contains == false
            {
                solutions.push((vec![packed_state], weight));
            }
        }
        return;
    }

    if state[row][col] > 3 // not assigned, try every value
    {
        for k in 0..4
        {
            state[row][col] = k;
            sh_constraints(state, (5 * row + col + 1) / 5, (5 * row + col + 1) % 5, sr, sc, br, bc, level, solutions, &cache);
            state[row][col] = 127;
        }
    } else { // skip that square
        sh_constraints(state, (5 * row + col + 1) / 5, (5 * row + col + 1) % 5, sr, sc, br, bc, level, solutions, &cache);
    }
}


// feasible so far, sophisticated pruning, checks correctly even if fully assigned
fn fsf(
    a: &[[usize; 5]; 5],
    sr: &[usize; 5],
    sc: &[usize; 5],
    br: &[usize; 5],
    bc: &[usize; 5]
) -> bool
{
    // rows
    for row in 0..5
    {
        let mut sum = 0;
        let mut bombs = 0;
        let mut ass = 0;

        for col in 0..5
        {
            if a[row][col] == 1 ||
                a[row][col] == 2 ||
                a[row][col] == 3
            {
                sum += a[row][col];
                ass += 1;
            }
            else if a[row][col] == 0
            {
                bombs += 1;
                ass += 1;
            }
        }

        if ass == 5 // no more assignments left --> needs to fit exactly
        {
            if sum != sr[row] || bombs != br[row]
            {
                return false;
            }
        } else { // just check if it's still within the limits

            // simple check
            if sum > sr[row] || bombs > br[row]
            {
                return false;
            }

            // more sophisticated check

            // bomb squares needed
            let bn = br[row] - bombs;

            // no more free squares e.g. more bombs left than squares available?
            if 5 < ass + bn
            {
                return false;
            }

            // free squares for points after taking away assigned and squares needed for bombs
            let fs = 5 - ass - bn;

            // sum needed, points left to reach sr
            let sn = sr[row] - sum;

            // two cases

            //  too little sn, will be exceeded even with 1's
            if sn < fs
            {
                return false;
            }

            // too much sn, can't be reached even with 3's
            if sn > fs * 3
            {
                return false;
            }
        }
    }

    // cols
    for col in 0..5
    {
        let mut sum = 0;
        let mut bombs = 0;
        let mut ass = 0;

        for row in 0..5
        {
            if a[row][col] == 1 ||
                a[row][col] == 2 ||
                a[row][col] == 3
            {
                sum += a[row][col];
                ass += 1;
            }
            else if a[row][col] == 0
            {
                bombs += 1;
                ass += 1;
            }
        }

        if ass == 5 // no more assignments left --> needs to fit exactly
        {
            if sum != sc[col] || bombs != bc[col]
            {
                return false;
            }
        } else { // just check if it's still within the limits

            // simple check
            if sum > sc[col] || bombs > bc[col]
            {
                return false;
            }

            // more sophisticated check

            // bomb squares needed
            let bn = bc[col] - bombs;

            // no more free squares e.g. more bombs left than squares available?
            if  5 < ass + bn
            {
                return false;
            }

            // free squares for points after taking away assigned and squares needed for bombs
            let fs = 5 - ass - bn;

            // sum needed, points left to reach sr
            let sn = sc[col] - sum;

            // two cases

            //  too little sn, will be exceeded even with 1's
            if sn < fs
            {
                return false;
            }

            // too much sn, can't be reached even with 3's
            if sn > fs * 3
            {
                return false;
            }
        }
    }

    true
}