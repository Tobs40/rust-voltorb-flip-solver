#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod csp_constraints;
mod packed;
mod csp_valid;
mod math;
mod level_constraints;
mod possible_boards;
mod search;
mod gui;
mod parsing;

use crate::gui::gui;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use crate::search::compute_win_chance_exact;
use std::time::Instant;
use std::io::{BufWriter, Write};
use std::collections::HashMap;
use crate::packed::{array_to_u64, u64_to_array, set_in_packed_state};
use crate::math::print_board;
use crate::level_constraints::get_constraint;
use crate::csp_valid::{count_valid_dumb, count_valid_smart};
use rayon::prelude::*;
use crate::csp_constraints::find_possible_boards;
use crate::possible_boards::accumulate_symbol_weights;

fn main() {
    gui();
}