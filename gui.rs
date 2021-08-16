use fltk::{app, button::*, enums::{Color, FrameType}, frame::Frame, prelude::*, window::Window, enums};
use std::sync::{Arc, Mutex, RwLock};
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::borrow::BorrowMut;
use fltk::enums::{Font, Event};
use ButtonMessage::*;
use std::fmt::format;
use std::cmp::{min, max};
use std::borrow::Borrow;
use std::time::Duration;
use crate::parsing::{string_to_level_and_constraints, file_to_puzzles};
use std::convert::TryInto;
use std::thread;
use std::collections::HashMap;
use crate::search::{compute_win_chance_exact, SearchResult};
use crate::packed::array_to_u64;
use fltk::misc::Tooltip;
use dashmap::DashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

// gui controlling the search
#[derive(Copy, Clone, Debug)]
pub enum ControlMessage
{
    Start,
    Stop,
    Constraints([usize;5], [usize;5], [usize;5], [usize;5], usize), // sr, sc, br, bc, level
    State([[usize;5];5]),
}

// the search reports it's results
#[derive(Copy, Clone, Debug)]
pub enum ReportMessage
{
    ConfirmStop, // no possible board for this constraints and level
SquareSymbols([[[f64;4];5];5]), // chances to bomb/1/2/3 for that square
SquareWinProb(usize, usize, f64), // win chance for that square: row, col, chance
FinishedSuccessfully(f64, u64, usize), // successful search, report win chance, comp. time, nodes
FinishedInconsistent,
FinishedTerminalState
}

#[derive(Copy, Clone, Debug)]
enum ButtonMessage
{
    SR(usize),
    SC(usize),
    BR(usize),
    BC(usize),
    Square(usize, usize),
    Level,
    Reset,
}

#[derive(Copy, Clone, Debug)]
enum ResizeMessage
{
    Resize,
}

fn update_buttons(
    state: &[[usize;5];5],
    level: &usize,
    sr: &[usize;5],
    sc: &[usize;5],
    br: &[usize;5],
    bc: &[usize;5],
    square_buttons: &mut Vec<Vec<Button>>,
    level_button: &mut Button,
    reset_button: &mut Button,
    sr_buttons: &mut Vec<Button>,
    sc_buttons: &mut Vec<Button>,
    br_buttons: &mut Vec<Button>,
    bc_buttons: &mut Vec<Button>,
    half_button_size: i32,
    window_width: i32
) -> ()
{
    let hbs = half_button_size;
    let fbs = hbs * 2;

    let small_font = hbs / 2;
    let large_font = hbs;

    let horizontal_shift = (window_width - 12 * hbs) / 2;

    for r in 0..5
    {
        for c in 0..5
        {
            square_buttons[r][c].set_pos(fbs * c as i32 + horizontal_shift, fbs * (r+1) as i32);
            square_buttons[r][c].set_size(fbs, fbs);

            square_buttons[r][c].set_label_size(large_font);

            if state[r][c] == 0
            {
                if square_buttons[r][c].label().len() > 0
                {
                    square_buttons[r][c].set_label_size(small_font);
                }
                else
                {
                    square_buttons[r][c].set_label("");
                }
            }
            else
            {
                square_buttons[r][c].set_label(&state[r][c].to_string());
            }
        }
    }

    level_button.set_pos(fbs * 5 + horizontal_shift, 0);
    level_button.set_size(fbs, hbs);

    level_button.set_label(&format!("Level {}", level));
    level_button.set_label_size(small_font);

    reset_button.set_pos(fbs * 5 + horizontal_shift, hbs);
    reset_button.set_size(fbs, hbs);

    reset_button.set_label("Reset");
    reset_button.set_label_size(small_font);

    for i in 0..5
    {
// pos and size
        sr_buttons[i].set_pos(fbs * 5 + horizontal_shift, fbs * (i+1) as i32);
        sr_buttons[i].set_size(fbs, hbs);

        br_buttons[i].set_pos(fbs * 5 + horizontal_shift, fbs * (i+1) as i32 + hbs);
        br_buttons[i].set_size(fbs, hbs);

        sc_buttons[i].set_pos(fbs * i as i32 + horizontal_shift, 0);
        sc_buttons[i].set_size(fbs, hbs);

        bc_buttons[i].set_pos(fbs * i as i32 + horizontal_shift, hbs);
        bc_buttons[i].set_size(fbs, hbs);

// label font size
        sr_buttons[i].set_label_size(small_font);
        sc_buttons[i].set_label_size(small_font);

        br_buttons[i].set_label_size(small_font);
        bc_buttons[i].set_label_size(small_font);

// label content
        sr_buttons[i].set_label(&sr[i].to_string());
        sc_buttons[i].set_label(&sc[i].to_string());

        br_buttons[i].set_label(&br[i].to_string());
        bc_buttons[i].set_label(&bc[i].to_string());
    }

    app::redraw();
}

fn tell_thread_start(
    to_thread: &Sender<ControlMessage>,
) -> ()
{
    println!("GUI: Telling thread to start");
    to_thread.send(ControlMessage::Start).expect("Failed to tell thread to start")
}

fn tell_thread_to_stop_and_wait_till_it_is_stopped(
    to_thread: &Sender<ControlMessage>,
    from_thread: &Receiver<ReportMessage>,
) -> ()
{
    println!("GUI: Signaling thread to stop");
    to_thread.send(ControlMessage::Stop).expect("Failed to signal Stop to thread");

    println!("GUI: Waiting for thread to confirm stop signal...");

    // wait for thread to confirm stop and eat the other messages while doing so
    loop {
        match from_thread.recv()
        {
            Ok(msg) => {
                match msg
                {
                    ReportMessage::ConfirmStop => {
                        println!("GUI: Thread confirmed stop signal with abort");
                        break;
                    },

                    ReportMessage::FinishedSuccessfully(_, _, _) => {
                        println!("GUI: Thread confirmed stop signal with successful search");
                        break;
                    },

                    ReportMessage::FinishedInconsistent => {
                        println!("GUI: Thread confirmed stop signal with inconsistent search");
                        break;
                    },

                    ReportMessage::FinishedTerminalState => {
                        println!("GUI: Thread confirmed stop signal with terminal state");
                        break;
                    },

                    msg => {
                        println!("GUI: Ignored {:?} signal from thread", msg);
                    }, // ignore other messages while waiting for the thread to stop
                }
            }

            Err(e) => {
                panic!("Error while waiting for thread to confirm stop signal: {}", e);
            }
        }

        println!("GUI: Still waiting for thread to confirm stop signal...");
    }
}

fn tell_thread_constraints(
    to_thread: &Sender<ControlMessage>,
    sr: &[usize;5],
    sc: &[usize;5],
    br: &[usize;5],
    bc: &[usize;5],
    level: usize,
) -> ()
{
    println!("GUI: Sending new constraints to thread");
    to_thread.send(
        ControlMessage::Constraints(
            sr.clone(), sc.clone(), br.clone(), bc.clone(), level
        )).expect("Failed to signal constraints to thread");
}

fn tell_thread_state(
    to_thread: &Sender<ControlMessage>,
    state: &[[usize;5];5],
) -> ()
{
    println!("GUI: Sending new state to thread");
    to_thread.send(
        ControlMessage::State(state.clone()
        )).expect("Failed to signal constraints to thread");
}

pub fn gui() -> ()
{
    //rayon::ThreadPoolBuilder::new().num_threads(12).build_global().unwrap();

    let (to_thread, from_gui) = unbounded();
    let to_thread_clone = to_thread.clone();

    let (to_gui, from_thread) = unbounded();

    let _ = thread::spawn(move || {

        // thread data structures, initialize with anything,
        // doesn't start calculating before the GUI tells it to anyway
        // and receives the puzzle data before that

        let mut org_packed_state = 0;
        let mut level = 1;
        let mut sr = [0;5];
        let mut sc = [0;5];
        let mut br = [0;5];
        let mut bc = [0;5];

        let cache_chances = DashMap::with_capacity(47_731_194);

        loop {
            match from_gui.recv()
            {
                Err(e) => panic!("Receiving in search thread failed: {}", e),

                Ok(k) => match k
                {
                    ControlMessage::Start => {

                        //println!("Thread: Starting search");

                        let search_result = compute_win_chance_exact(
                            org_packed_state, &sr, &sc, &br, &bc, level,
                            &cache_chances, &from_gui, &to_gui, &to_thread_clone);

                        match search_result
                        {
                            SearchResult::SuccessfulSearchWithInfo(prob, time, size) => {
                                //println!("Thread: Search finished successfully, signalling successful search");
                                to_gui.send(ReportMessage::FinishedSuccessfully(prob, time.round() as u64, size))
                                    .expect("Sending finished failed");
                            }

                            SearchResult::TerminalState => {
                                //println!("Thread: Got terminal state as root state");
                                to_gui.send(ReportMessage::FinishedTerminalState)
                                    .expect("Sending finished failed");
                            }

                            SearchResult::SuccessfulSearch(p) => {
                                panic!("{:?} is not meant to be used outside of search thread",
                                       SearchResult::SuccessfulSearch(p));
                            }

                            SearchResult::Aborted => {
                                //println!("Thread: Search has been aborted, sending stop confirmation");
                                to_gui.send(ReportMessage::ConfirmStop)
                                    .expect("Sending stop confirmation failed");
                            }

                            SearchResult::InconsistentPuzzle => {
                                //println!("Thread: Puzzle is inconsistent, telling GUI");
                                to_gui.send(ReportMessage::FinishedInconsistent)
                                    .expect("Failed to signal inconsistency to GUI");
                            }
                        }
                    },

                    ControlMessage::Stop => {
                        //println!("Thread: Got stop signal (wasn't searching). Sending confirmation anyway.");
                        to_gui.send(ReportMessage::ConfirmStop)
                            .expect("Sending stop confirmation failed");
                    },

                    ControlMessage::Constraints(sr_, sc_, br_, bc_, level_) => {
                        //println!("Thread: Received new constraints, clearing the cache");
                        sr = sr_;
                        sc = sc_;
                        br = br_;
                        bc = bc_;
                        level = level_;
                        cache_chances.clear();
                    },

                    ControlMessage::State(state_) => {
                        //println!("Thread: Received new state");
                        org_packed_state = array_to_u64(&state_);
                    },
                }
            }
        }
    }
    );

    const TITLE: &str = "Voltorb Flip Solver by Tobi :)";
    const TITLE_INCONSISTENT: &str = "Enter valid constraints and level";
    const TITLE_CALCULATING_POSSIBLE_BOARDS: &str = "Calculating possible boards...";
    const TITLE_CALCULATING_WIN_CHANCE: &str = "Calculating win chances...";
    const TITLE_TERMINAL_STATE: &str = "You won :)";

    // data structures which are set through the buttons
    let mut state = [[0; 5]; 5];
    let (_, mut sr, mut sc, mut br, mut bc, mut level) = {
        if true
        {
            // load first puzzle of examples.txt without requiring the file
            string_to_level_and_constraints("31010-11203-10110-11211-11101 1")
        }
        else {
            // 348 in examples.txt is a hard one
            file_to_puzzles("examples.txt")[348 -1]
        }
    };

    let mut half_button_size = 50;

    Tooltip::set_delay(0.0);
    Tooltip::set_hoverdelay(0.0);
    Tooltip::set_font_size(25);
    Tooltip::set_font(Font::Helvetica);

    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    app::background(0, 0, 0);
    app::set_font(Font::Helvetica);
    app::set_visible_focus(false);

    let (sender_app, receiver_app) = app::channel::<ButtonMessage>();

    let mut window = Window::default()
        .with_size(12 * half_button_size, 12 * half_button_size)
        .center_screen()
        .with_label(TITLE);

    // Square buttons
    let mut square_buttons = Vec::with_capacity(5);

    for r in 0..5
    {
        let mut v = Vec::with_capacity(5);
        for c in 0..5
        {
            let mut b = Button::default();

            let r = r as usize;
            let c = c as usize;

            b.set_color(Color::White);
            b.set_selection_color(Color::White);
            b.emit(sender_app, Square(r, c));

            v.push(b);
        }
        square_buttons.push(v);
    }

    // row points
    let mut sr_buttons = Vec::with_capacity(5);

    for r in 0..5
    {
        let mut b = Button::default();

        b.set_color(Color::Green);
        b.set_selection_color(Color::Green);
        b.emit(sender_app, SR(r as usize));

        sr_buttons.push(b);
    }

    // row bombs
    let mut br_buttons = Vec::with_capacity(5);

    for r in 0..5
    {
        let mut b = Button::default();

        b.set_color(Color::Red);
        b.set_selection_color(Color::Red);
        b.emit(sender_app, BR(r as usize));

        br_buttons.push(b);
    }

    // col points
    let mut sc_buttons = Vec::with_capacity(5);

    for c in 0..5
    {
        let mut b = Button::default();

        b.set_color(Color::Green);
        b.set_selection_color(Color::Green);
        b.emit(sender_app, SC(c as usize));

        sc_buttons.push(b);
    }

    // col bombs
    let mut bc_buttons = Vec::with_capacity(5);

    for c in 0..5
    {
        let mut b = Button::default();

        b.set_color(Color::Red);
        b.set_selection_color(Color::Red);
        b.emit(sender_app, BC(c as usize));

        bc_buttons.push(b);
    }

    // level button
    let mut level_button = Button::default();

    level_button.set_color(Color::Cyan);
    level_button.set_selection_color(Color::Cyan);
    level_button.emit(sender_app, Level);

    // reset button
    let mut reset_button = Button::default();

    reset_button.set_color(Color::DarkGreen);
    reset_button.set_selection_color(Color::Green);
    reset_button.emit(sender_app, Reset);

    window.make_resizable(true);
    window.set_color(Color::DarkCyan);

    window.end();
    window.show();

    update_buttons(
        &state,
        &level,
        &sr,
        &sc,
        &br,
        &bc,
        &mut square_buttons,
        &mut level_button,
        &mut reset_button,
        &mut sr_buttons,
        &mut sc_buttons,
        &mut br_buttons,
        &mut bc_buttons,
        half_button_size,
        window.width(),
    );

    let (sender_resize, receiver_resize) = unbounded();
    window.handle(move |w, ev| match ev {

        Event::Resize => {
            sender_resize.send(ResizeMessage::Resize).expect("Couldn't send resize message");
            true
        }

        _ => false,
    });

    let mut old_width = window.width();
    let mut old_height = window.height();

    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
    tell_thread_state(&to_thread, &state);
    tell_thread_start(&to_thread);

    let mut symbol_probs: Option<[[[f64;4];5];5]> = None;
    let mut win_chances = [[None;5];5];

    while app::wait_for(1.0/60.0)
        .expect("Crashed while waiting for something to happen")
    {
        let mouse_x = app::event_x();
        let mouse_y = app::event_y();

        let pressed_number = {
            let mut tmp = None;
            for i in 0..=9
            {
                let c = char::from_digit(i as u32, 10).unwrap();
                if app::event_key_down(fltk::enums::Key::from_char(c))
                {
                    tmp = Some(i);
                }
            }
            tmp
        };

        // process resizing
        match receiver_resize.try_recv()
        {
            Ok(ResizeMessage::Resize) => {
                let w = window.width();
                let h = window.height();

                // resized? Sometimes you get fake resize messages here for some reason
                if w != old_width || h != old_height
                {
                    half_button_size = {
                        if window.x() == 0
                        {
                            h / 12
                        } else {
                            let only_width_changed = (w != old_width) && (h == old_height);
                            let only_height_changed = (w == old_width) && (h != old_height);
                            let both_changed = (w != old_width) && (h != old_height);

                            if only_width_changed
                            {
                                w / 12
                            } else if only_height_changed
                            {
                                h / 12
                            } else {
                                min(w, h) / 12
                            }
                        }
                    };

                    let s = half_button_size * 12;

                    // only resize if it's not "full screen"
                    if window.x() != 0
                    {
                        window.resize(window.x(), window.y(), s, s);
                    }

                    //println!("GUI: Got resize message, resized from {}x{} to {}x{}", old_width, old_height, s, s);

                    update_buttons(
                        &state,
                        &level,
                        &sr,
                        &sc,
                        &br,
                        &bc,
                        &mut square_buttons,
                        &mut level_button,
                        &mut reset_button,
                        &mut sr_buttons,
                        &mut sc_buttons,
                        &mut br_buttons,
                        &mut bc_buttons,
                        half_button_size,
                        window.width(),
                    );

                    old_width = window.width();
                    old_height = window.height();
                }
                else {
                    //println!("GUI: Got resize message but size is same as before")
                }
            },
            _ => (),
        }

        // process messages from search thread
        if let Ok(msg) = from_thread.try_recv()
        {
            match msg
            {
                ReportMessage::FinishedInconsistent => {
                    //println!("GUI: Puzzle is inconsistent");
                    window.set_label(TITLE_INCONSISTENT);
                    for r in 0..5
                    {
                        for c in 0..5
                        {
                            square_buttons[r][c].set_color(Color::White);
                        }
                    }
                },

                ReportMessage::FinishedTerminalState => {
                    //println!("GUI: Root state was terminal state");
                    window.set_label(TITLE_TERMINAL_STATE);
                },

                ReportMessage::FinishedSuccessfully(prob, time, nodes) => {
                    //println!("GUI: Search finished successfully");
                    if time == 0
                    {
                        window.set_label(&format!("Win chance: {:.2}%", prob * 100.0));
                    }
                    else
                    {
                        window.set_label(&format!("Win chance: {:.2}% in {} seconds, {} nodes", prob * 100.0, time, nodes));
                    }


                    let mut exists_safe_and_useful = false;
                    let mut best_win_chance = prob;

                    let sp = symbol_probs.unwrap();

                    for r in 0..5
                    {
                        for c in 0..5
                        {
                            if state[r][c] == 0
                            {
                                let safe_and_useful =
                                    sp[r][c][0] == 0.0 && sp[r][c][2] + sp[r][c][3] > 0.0;
                                if safe_and_useful
                                {
                                    exists_safe_and_useful = true;
                                }
                            }

                            if let Some(p) = win_chances[r][c]
                            {
                                if p > best_win_chance
                                {
                                    best_win_chance = p;
                                }
                            }
                        }
                    }

                    // mark square with best win chance
                    if exists_safe_and_useful == false
                    {
                        for r in 0..5
                        {
                            for c in 0..5
                            {
                                if let Some(p) = win_chances[r][c]
                                {
                                    if best_win_chance - 1e-5 < p
                                    {
                                        square_buttons[r][c].set_color(Color::Blue);
                                        square_buttons[r][c].set_label_size(half_button_size / 2);
                                    }
                                    else {
                                        square_buttons[r][c].set_color(Color::color_average(Color::Cyan, Color::White, 0.3));
                                        square_buttons[r][c].set_label_size(half_button_size / 2);
                                    }
                                }
                            }
                        }
                    }
                }

                ReportMessage::SquareWinProb(r, c, p) => {

                    //println!("GUI: Received win prob for square ({}, {}), it's {}", r, c, p);

                    let sp = symbol_probs.unwrap();

                    win_chances[r][c] = Some(p);

                    square_buttons[r][c].set_tooltip(
                        &format!("Bomb: {:.2}%\nOne: {:.2}%\nTwo: {:.2}%\n\
                        Three: {:.2}%\nChance: {:.2}%",
                                 sp[r][c][0] * 100.0, sp[r][c][1] * 100.0,
                                 sp[r][c][2] * 100.0, sp[r][c][3] * 100.0,
                                 p * 100.0
                        )
                    );

                    let mut exists_safe_and_useful = false;
                    let mut best_win_chance = p;

                    for r in 0..5
                    {
                        for c in 0..5
                        {
                            if state[r][c] == 0
                            {
                                let safe_and_useful =
                                    sp[r][c][0] == 0.0 && sp[r][c][2] + sp[r][c][3] > 0.0;
                                if safe_and_useful
                                {
                                    exists_safe_and_useful = true;
                                }
                            }

                            if let Some(p) = win_chances[r][c]
                            {
                                if p > best_win_chance
                                {
                                    best_win_chance = p;
                                }
                            }
                        }
                    }

                    // mark square with best win chance
                    if exists_safe_and_useful == false
                    {
                        for r in 0..5
                        {
                            for c in 0..5
                            {
                                if let Some(p) = win_chances[r][c]
                                {
                                    square_buttons[r][c].set_color(Color::color_average(Color::Cyan, Color::White, 0.3));
                                    square_buttons[r][c].set_label_size(half_button_size / 2);
                                }
                            }
                        }
                    }

                    square_buttons[r][c].set_label(&format!("{:.0}%", p * 100.0));
                },

                ReportMessage::SquareSymbols(sp) => {
                    //println!("GUI: Received symbol probs for all squares");
                    symbol_probs = Some(sp);
                    window.set_label(TITLE_CALCULATING_WIN_CHANCE);
                    for r in 0..5
                    {
                        for c in 0..5
                        {
                            if state[r][c] == 0
                            {
                                let color = {
                                    if sp[r][c][2] + sp[r][c][3] == 0.0 // useless?
                                    {
                                        Color::from_rgb(50, 50, 50) // gray?
                                    }
                                    else {
                                        if sp[r][c][0] == 0.0 // safe? (and useful)
                                        {
                                            Color::Yellow
                                        }
                                        else {
                                            Color::White
                                        }
                                    }
                                };

                                square_buttons[r][c].set_color(color);
                            }

                            square_buttons[r][c].set_tooltip(
                                &format!("Bomb: {:.2}%\nOne: {:.2}%\nTwo: {:.2}%\nThree: {:.2}%",
                                         sp[r][c][0] * 100.0, sp[r][c][1] * 100.0,
                                         sp[r][c][2] * 100.0, sp[r][c][3] * 100.0,
                                )
                            );
                            Tooltip::enable(true);
                        }
                    }
                }

                ReportMessage::ConfirmStop => {
                    //println!("GUI: Got a (probably redundant) stop signal confirmation in the main loop, ignoring it");
                }
            }

            update_buttons(
                &state,
                &level,
                &sr,
                &sc,
                &br,
                &bc,
                &mut square_buttons,
                &mut level_button,
                &mut reset_button,
                &mut sr_buttons,
                &mut sc_buttons,
                &mut br_buttons,
                &mut bc_buttons,
                half_button_size,
                window.width(),
            );
            //app::redraw();
        }

        // process button input
        if let Some(msg) = receiver_app.recv()
        {
            match msg
            {
                Square(r, c) => {

                    match pressed_number
                    {
                        None => {
                            state[r][c] = (state[r][c] + 1) % 4;
                        }

                        Some(n) => {
                            if n <= 3
                            {
                                state[r][c] = n;
                            }
                        }
                    }

                    //println!("GUI: Square ({}, {}) was updated to {}", r, c, state[r][c]);

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_state(&to_thread, &state);
                    tell_thread_start(&to_thread);

                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                },

                Level => {
                    match pressed_number
                    {
                        None => {
                            level = level % 8 + 1;
                        }

                        Some(n) => {
                            if n >= 1 && n <= 8
                            {
                                level = n;
                            }
                        }
                    }

                    //println!("GUI: Level was updated to {}", level);

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
                    tell_thread_start(&to_thread);
                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                },

                Reset => {

                    //println!("GUI: Reset state, level and constraints");

                    for r in 0..5
                    {
                        for c in 0..5
                        {
                            state[r][c] = 0;
                        }
                    }

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_state(&to_thread, &state);
                    tell_thread_start(&to_thread);

                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                }

                SR(r) => {

                    match pressed_number
                    {
                        None => {
                            sr[r] = (sr[r] + 1) % 16;
                        }

                        Some(n) => {
                            sr[r] = n;
                        }
                    }

                    //println!("GUI: Sum of points of row {} updated to {}", r, sr[r]);

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
                    tell_thread_start(&to_thread);
                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                },

                SC(c) => {

                    match pressed_number
                    {
                        None => {
                            sc[c] = (sc[c] + 1) % 16;
                        }

                        Some(n) => {
                            sc[c] = n;
                        }
                    }

                    //println!("GUI: Sum of points of col {} updated to {}", c, sc[c]);

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
                    tell_thread_start(&to_thread);
                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                },

                BR(r) => {

                    match pressed_number
                    {
                        None => {
                            br[r] = (br[r] + 1) % 6;
                        }

                        Some(n) => {
                            if n <= 5
                            {
                                br[r] = n;
                            }
                        }
                    }

                    //println!("GUI: Bomb count of row {} updated to {}", r, br[r]);

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
                    tell_thread_start(&to_thread);
                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                },

                BC(c) => {

                    match pressed_number
                    {
                        None => {
                            bc[c] = (bc[c] + 1) % 6;
                        }

                        Some(n) => {
                            if n <= 5
                            {
                                bc[c] = n;
                            }
                        }
                    }

                    //println!("GUI: Bomb count of col {} updated to {}", c, bc[c]);

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
                    tell_thread_start(&to_thread);
                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                }
            }

            Tooltip::disable();
            symbol_probs = None;
            win_chances = [[None;5];5];

            for r in 0..5
            {
                for c in 0..5
                {
                    square_buttons[r][c].set_label("");
                    square_buttons[r][c].set_color(Color::White);
                }
            }

            update_buttons(
                &state,
                &level,
                &sr,
                &sc,
                &br,
                &bc,
                &mut square_buttons,
                &mut level_button,
                &mut reset_button,
                &mut sr_buttons,
                &mut sc_buttons,
                &mut br_buttons,
                &mut bc_buttons,
                half_button_size,
                window.width(),
            );
        }
    }
}