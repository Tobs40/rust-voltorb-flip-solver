use fltk::{app, button::*, group, enums::{Color, FrameType}, frame::Frame, prelude::*, window::Window, enums, menu};
use std::sync::{Arc, Mutex, RwLock};
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::borrow::BorrowMut;
use fltk::enums::{Font, Event, Shortcut};
use ButtonMessage::*;
use std::fmt::format;
use std::cmp::{min, max};
use std::borrow::Borrow;
use std::time::Duration;
use crate::parsing::{string_to_level_and_constraints, hardest_5, examples_357};
use std::convert::TryInto;
use std::{thread, process};
use std::collections::HashMap;
use crate::search::{compute_win_chance_exact, SearchResult};
use crate::packed::{array_to_u64, coins_of_state, u64_to_array};
use fltk::misc::Tooltip;
use dashmap::DashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use fltk::menu::MenuFlag;
use fltk::window::SingleWindow;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::math::count_assigned_packed;
use dashmap::mapref::multiple::RefMulti;
use fltk::valuator::{Counter, CounterType};

// gui controlling the search
#[derive(Copy, Clone, Debug)]
pub enum ControlMessage
{
    Start,
    Stop,
    Constraints([usize;5], [usize;5], [usize;5], [usize;5], usize), // sr, sc, br, bc, level
State([[usize;5];5]),
    Mode(SearchMode),
    Threads(usize),
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
pub enum ButtonMessage
{
    SR(usize),
    SC(usize),
    BR(usize),
    BC(usize),
    Square(usize, usize),
    Level,
    Reset,
    Mode(SearchMode),
    Threads,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SearchMode
{
    WinChance,
    WinEight,
    SurviveNextMove,
    SurviveLevel,
    SurviveEight,
    Coins,
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
    info!("GUI: Telling thread to start");
    to_thread.send(ControlMessage::Start).expect("Failed to tell thread to start")
}

fn tell_thread_to_stop_and_wait_till_it_is_stopped(
    to_thread: &Sender<ControlMessage>,
    from_thread: &Receiver<ReportMessage>,
) -> ()
{
    info!("GUI: Signaling thread to stop");
    to_thread.send(ControlMessage::Stop).expect("Failed to signal Stop to thread");

    info!("GUI: Waiting for thread to confirm stop signal...");

    // wait for thread to confirm stop and eat the other messages while doing so
    loop {
        match from_thread.recv()
        {
            Ok(msg) => {
                match msg
                {
                    ReportMessage::ConfirmStop => {
                        info!("GUI: Thread confirmed stop signal with abort");
                        break;
                    },

                    ReportMessage::FinishedSuccessfully(_, _, _) => {
                        info!("GUI: Thread confirmed stop signal with successful search");
                        break;
                    },

                    ReportMessage::FinishedInconsistent => {
                        info!("GUI: Thread confirmed stop signal with inconsistent search");
                        break;
                    },

                    ReportMessage::FinishedTerminalState => {
                        info!("GUI: Thread confirmed stop signal with terminal state");
                        break;
                    },

                    msg => {
                        info!("GUI: Ignored {:?} signal from thread while waiting to stop", msg);
                    }, // ignore other messages while waiting for the thread to stop
                }
            }

            Err(e) => {
                panic!("Error while waiting for thread to confirm stop signal: {}", e);
            }
        }

        info!("GUI: Still waiting for thread to confirm stop signal...");
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
    info!("GUI: Sending new constraints to thread");
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
    info!("GUI: Sending new state to thread");
    to_thread.send(
        ControlMessage::State(state.clone()
        )).expect("Failed to signal state to thread");
}

fn tell_thread_mode(
    to_thread: &Sender<ControlMessage>,
    mode: SearchMode,
) -> ()
{
    info!("GUI: Sending new mode to thread");
    to_thread.send(
        ControlMessage::Mode(mode)).expect("Failed to signal mode to thread");
}

fn tell_thread_threads(
    to_thread: &Sender<ControlMessage>,
    threads: usize,
) -> ()
{
    info!("GUI: Sending new thread count to thread");
    to_thread.send(
        ControlMessage::Threads(threads))
        .expect("Failed to signal thread number to thread");
}

pub fn gui() -> ()
{
    // 25 squares in the initial position, the number of active threads is controlled otherwise
    rayon::ThreadPoolBuilder::new().num_threads(25).build_global().unwrap();

    let recommended_threads = max(1, num_cpus::get() - 2); // two logical cores to not lag
    let mut threads = recommended_threads + 2;

    const TITLE_CALCULATING_POSSIBLE_BOARDS: &str = "Calculating possible boards...";
    const DESCRIPTION_WIN: &str = "Maximizes the chance to find all 2 and 3, doesn't care whether you die now or later. Useful for reaching the next level.";
    const DESCRIPTION_WIN_EIGHT: &str = "Maximizes the chance to find all 2 and 3 AND have at least 8 cards face-up when winning. Useful for reaching level 8.";
    const DESCRIPTION_SURVIVE_NEXT_MOVE: &str = "Picks the square with the lowest chance to be a bomb from all squares that could be a 2 or 3. Useful for very slow PC's and if you're scared of falling back to low levels.";
    const DESCRIPTION_SURVIVE_LEVEL: &str = "Maximizes the chance to flip #level cards without loosing (=making sure that the level doesn't decrease).";
    const DESCRIPTION_SURVIVE_EIGHT: &str = "Maximizes the chance to flip 8 cards without loosing. Useful for reaching level 8.";
    const DESCRIPTION_COINS: &str = "Maximizes the expected number of coins. Useful if you just need some more.";

    let (to_thread, from_gui) = unbounded();
    let to_thread_clone = to_thread.clone();

    let (to_gui, from_thread) = unbounded();

    let _ = thread::spawn(move || {

        // thread data structures, initialize with anything,
        // doesn't start calculating before the GUI tells it to anyway
        // and receives the puzzle data before that

        let mut org_packed_state = None;
        let mut level = None;
        let mut sr = None;
        let mut sc = None;
        let mut br = None;
        let mut bc = None;
        let mut mode: Option<SearchMode> = None;
        let mut threads = None;

        // map of small partial maps for each mode
        let caches: DashMap<SearchMode, DashMap<u64, f64>> = DashMap::new(); // TODO clear caches upon constraint change
        let big_cache = DashMap::with_capacity(111_744_155);

        loop {
            match from_gui.recv()
            {
                Err(e) => panic!("Receiving in search thread failed: {}", e),

                Ok(k) => match k
                {
                    ControlMessage::Start => {

                        info!("Thread: Starting search");

                        if let Some(entry) = caches.get(&mode.unwrap())
                        {
                            let mini_cache = entry.value();
                            for entry in mini_cache
                            {
                                big_cache.insert(*entry.key(), *entry.value());
                            }
                        }

                        let search_result = compute_win_chance_exact(
                            org_packed_state.unwrap(),
                            &sr.unwrap(),
                            &sc.unwrap(),
                            &br.unwrap(),
                            &bc.unwrap(),
                            level.unwrap(),
                            &big_cache, &from_gui, &to_gui, &to_thread_clone,
                            mode.unwrap(),
                            threads.unwrap());

                        match search_result
                        {
                            SearchResult::SuccessfulSearchWithInfo(prob, time, size) => {

                                info!("Thread: Search finished successfully, signalling successful search");
                                to_gui.send(ReportMessage::FinishedSuccessfully(prob, time.round() as u64, size))
                                    .expect("Sending finished failed");
                            }

                            SearchResult::TerminalState => {
                                info!("Thread: Got terminal state as root state");
                                to_gui.send(ReportMessage::FinishedTerminalState)
                                    .expect("Sending finished failed");
                            }

                            SearchResult::SuccessfulSearch(p) => {
                                panic!("{:?} is not meant to be used outside of search thread",
                                       SearchResult::SuccessfulSearch(p));
                            }

                            SearchResult::Aborted => {
                                info!("Thread: Search has been aborted, sending stop confirmation");
                                to_gui.send(ReportMessage::ConfirmStop)
                                    .expect("Sending stop confirmation failed");
                            }

                            SearchResult::InconsistentPuzzle => {
                                info!("Thread: Puzzle is inconsistent, telling GUI");
                                to_gui.send(ReportMessage::FinishedInconsistent)
                                    .expect("Failed to signal inconsistency to GUI");
                            }
                        }

                        // also keep the cache of aborted searches around

                        // no cache for this mode? Create a new one
                        if !caches.contains_key(&mode.unwrap())
                        {
                            caches.insert(mode.unwrap(), DashMap::with_capacity(16_384));
                        }

                        // get the cache for the current mode
                        if let Some(entry) = caches.get(&mode.unwrap())
                        {
                            let mini_cache = entry.value();
                            // copy the most important content into that mini_cache
                            // big_cache is the cache the algorithm uses
                            for entry in &big_cache
                            {
                                let board = *entry.key();
                                let value = *entry.value();

                                let assigned = count_assigned_packed(board);

                                if assigned <= 3
                                {
                                    mini_cache.insert(board, value);
                                }
                            }
                        }
                    },

                    ControlMessage::Stop => {
                        info!("Thread: Got stop signal but wasn't searching");
                        to_gui
                            .send(ReportMessage::ConfirmStop)
                            .expect("Failed to confirm stop to GUI");
                    },

                    ControlMessage::Constraints(sr_, sc_, br_, bc_, level_) => {
                        info!("Thread: Received new constraints, clearing the caches");
                        sr = Some(sr_);
                        sc = Some(sc_);
                        br = Some(br_);
                        bc = Some(bc_);
                        level = Some(level_);
                        caches.clear();
                        big_cache.clear();
                    },

                    ControlMessage::State(state_) => {
                        info!("Thread: Received new state");
                        org_packed_state = Some(array_to_u64(&state_));
                    },

                    ControlMessage::Mode(m) => {
                        if mode == None || mode.unwrap() != m // has changed
                        {
                            info!("Thread: Setting mode to {:?}", m);
                            big_cache.clear();
                            mode = Some(m);
                        }
                        else
                        {
                            info!("Thread: Mode is already {:?}", m);
                        }
                    }

                    ControlMessage::Threads(t) => {
                        if threads == None || threads.unwrap() != t
                        {
                            threads = Some(t);
                            info!("Thread: Using {} threads for calculation", t);
                        }
                        else
                        {
                            info!("Thread: Already using {} threads for calculation", t);
                        }
                    }
                }
            }
        }
    }
    );

    // data structures which are set through the buttons
    let mut state = [[0; 5]; 5];
    let (_, mut sr, mut sc, mut br, mut bc, mut level, _state) = {
        examples_357()[1-1] // puzzle number 1
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
        .with_label("Voltorb Flip Solver by Tobias Völk :)");

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
            b.emit(sender_app, ButtonMessage::Square(r, c));

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
        b.emit(sender_app, ButtonMessage::SR(r as usize));

        sr_buttons.push(b);
    }

    // row bombs
    let mut br_buttons = Vec::with_capacity(5);

    for r in 0..5
    {
        let mut b = Button::default();

        b.set_color(Color::Red);
        b.set_selection_color(Color::Red);
        b.emit(sender_app, ButtonMessage::BR(r as usize));

        br_buttons.push(b);
    }

    // col points
    let mut sc_buttons = Vec::with_capacity(5);

    for c in 0..5
    {
        let mut b = Button::default();

        b.set_color(Color::Green);
        b.set_selection_color(Color::Green);
        b.emit(sender_app, ButtonMessage::SC(c as usize));

        sc_buttons.push(b);
    }

    // col bombs
    let mut bc_buttons = Vec::with_capacity(5);

    for c in 0..5
    {
        let mut b = Button::default();

        b.set_color(Color::Red);
        b.set_selection_color(Color::Red);
        b.emit(sender_app, ButtonMessage::BC(c as usize));

        bc_buttons.push(b);
    }

    // level button
    let mut level_button = Button::default();

    level_button.set_color(Color::Cyan);
    level_button.set_selection_color(Color::Cyan);
    level_button.emit(sender_app, ButtonMessage::Level);

    // reset button
    let mut reset_button = Button::default();

    reset_button.set_color(Color::DarkGreen);
    reset_button.set_selection_color(Color::Green);
    reset_button.emit(sender_app, ButtonMessage::Reset);

    window.make_resizable(true);
    window.set_color(Color::DarkCyan);

    window.end();
    window.show();

    let mut window2 = Window::default()
        .with_size(400, 200)
        .center_screen()
        .with_label("Algorithm and Threads");

    window2.make_resizable(true);

    let mut pack = group::Group::new(0, 0, 400, 200, "");
    pack.end();
    pack.set_type(group::PackType::Vertical);

    let mut menu_choice = menu::Choice::default();

    menu_choice.set_pos(0, 0);
    menu_choice.set_size(400, 30);
    menu_choice.set_text_size(20);
    menu_choice.set_color(Color::White);

    // make sure the first one is the same as the one sent to the thread
    menu_choice.add_emit("SurviveNextMove", Shortcut::None, MenuFlag::Normal, sender_app, ButtonMessage::Mode(SearchMode::SurviveNextMove));
    menu_choice.add_emit("Win", Shortcut::None, MenuFlag::Normal, sender_app, ButtonMessage::Mode(SearchMode::WinChance));
    menu_choice.add_emit("Win + SurviveEight", Shortcut::None, MenuFlag::Normal, sender_app, ButtonMessage::Mode(SearchMode::WinEight));
    menu_choice.add_emit("SurviveLevel", Shortcut::None, MenuFlag::Normal, sender_app, ButtonMessage::Mode(SearchMode::SurviveLevel));
    menu_choice.add_emit("SurviveEight", Shortcut::None, MenuFlag::Normal, sender_app, ButtonMessage::Mode(SearchMode::SurviveEight));
    menu_choice.add_emit("Coins", Shortcut::None, MenuFlag::Normal, sender_app, ButtonMessage::Mode(SearchMode::Coins));

    let mut mode = SearchMode::SurviveNextMove;
    menu_choice.set_item(&menu_choice.at(0).unwrap());

    let mut text_display = fltk::text::TextDisplay::default();

    text_display.set_buffer(fltk::text::TextBuffer::default());
    text_display.buffer().unwrap().set_text(DESCRIPTION_SURVIVE_NEXT_MOVE);
    text_display.wrap_mode(fltk::text::WrapMode::AtBounds, 0);

    text_display.set_pos(0, 30);
    text_display.set_size(400, 140);
    text_display.set_text_size(20);
    text_display.set_color(Color::White);

    let mut counter = Counter::new(0, 170, 400, 30, "MyCounter");
    counter.set_type(CounterType::Simple);

    counter.set_color(Color::White);
    counter.set_label_size(20);

    counter.set_precision(0);
    counter.set_minimum(1.0); // pausing is ok too

    let logical_cores = num_cpus::get();
    counter.set_maximum(logical_cores as f64);

    counter.set_value(threads as f64);

    counter.emit(sender_app, ButtonMessage::Threads);

    pack.add(&menu_choice);
    pack.add_resizable(&text_display);
    pack.add(&counter);

    window2.end();
    window2.show();

    window.visible_focus(false);
    window2.visible_focus(false);

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

    let running = Arc::new(AtomicBool::new(true));
    let running_window = running.clone();
    let running_window2 = running.clone();

    // close everything as soon as one window is gone
    window.set_callback(move |_| {
        info!("GUI: Main window has been closed, exiting soon...");
        running_window.store(false, Ordering::SeqCst);
    });

    window2.set_callback(move |_| {
        info!("GUI: Secondary window has been closed, exiting soon...");
        running_window2.store(false, Ordering::SeqCst);
    });

    let mut old_width = window.width();
    let mut old_height = window.height();

    // already is stopped
    tell_thread_mode(&to_thread, mode);
    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
    tell_thread_state(&to_thread, &state);
    tell_thread_threads(&to_thread, threads);
    tell_thread_start(&to_thread);

    let mut symbol_probs: Option<[[[f64;4];5];5]> = None;
    let mut probs = [[None;5];5];

    while app::wait_for(1.0/600.0).expect("Crashed while waiting for something to happen")
    {
        if running.load(Ordering::SeqCst) == false
        {
            break; // exit program
        }

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

                    info!("GUI: Got resize message, resized from {}x{} to {}x{}", old_width, old_height, s, s);

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
                    info!("GUI: Got resize message but size is same as before")
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
                    info!("GUI: Puzzle is inconsistent");
                    window.set_label("Invalid constraints/cards/level");
                    for r in 0..5
                    {
                        for c in 0..5
                        {
                            square_buttons[r][c].set_color(Color::White);
                        }
                    }
                },

                ReportMessage::FinishedTerminalState => {
                    info!("GUI: Root state was terminal state");
                    window.set_label(
                        &format!(
                            "You won, yey, {} coins", coins_of_state(array_to_u64(&state))
                        )
                    );
                },

                ReportMessage::FinishedSuccessfully(val, time, nodes) => {
                    info!("GUI: Search finished successfully");
                    match mode
                    {
                        SearchMode::WinChance => window.set_label(&format!("Win: {:.2}%", val * 100.0)),

                        SearchMode::WinEight => window.set_label(&format!("Win and uncover at least 8 cards: {:.2}%", val * 100.0)),

                        SearchMode::SurviveLevel => window.set_label(&format!("Survive {} moves: {:.2}%", level, val * 100.0)),

                        SearchMode::SurviveEight => window.set_label(&format!("Survive 8 moves: {:.2}%", val * 100.0)),

                        SearchMode::SurviveNextMove => window.set_label(&format!("Survive next move: {:.2}%", val * 100.0)),

                        SearchMode::Coins => window.set_label(&format!("Expected coins: {:.2}", val)),
                    }

                    let mut exists_safe_and_useful = false;
                    let mut best_win_chance = val;

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

                            if let Some(p) = probs[r][c]
                            {
                                if p > best_win_chance
                                {
                                    best_win_chance = p;
                                }
                            }
                        }
                    }

                    // mark square with best win chance
                    if exists_safe_and_useful == false ||
                        mode == SearchMode::WinEight ||
                        mode == SearchMode::SurviveLevel ||
                        mode == SearchMode::SurviveEight
                    {
                        for r in 0..5
                        {
                            for c in 0..5
                            {
                                if let Some(p) = probs[r][c]
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

                ReportMessage::SquareWinProb(row, col, val) => {
                    info!("GUI: Received value for square ({}, {}), it's {}", row, col, val);

                    let sp = symbol_probs.unwrap();
                    probs[row][col] = Some(val);

                    if mode != SearchMode::Coins
                    {
                        square_buttons[row][col].set_tooltip(
                            &format!("Bomb: {:.2}%\nOne: {:.2}%\nTwo: {:.2}%\n\
                        Three: {:.2}%\n{:.2}%",
                                     sp[row][col][0] * 100.0, sp[row][col][1] * 100.0,
                                     sp[row][col][2] * 100.0, sp[row][col][3] * 100.0,
                                     val * 100.0
                            )
                        );
                    }
                    else {
                        square_buttons[row][col].set_tooltip(
                            &format!("Bomb: {:.2}%\nOne: {:.2}%\nTwo: {:.2}%\n\
                        Three: {:.2}%\n{:.2}",
                                     sp[row][col][0] * 100.0, sp[row][col][1] * 100.0,
                                     sp[row][col][2] * 100.0, sp[row][col][3] * 100.0,
                                     val
                            )
                        );
                    }

                    let mut exists_safe_and_useful = false;
                    let mut best_win_chance = val;

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

                            if let Some(p) = probs[r][c]
                            {
                                if p > best_win_chance
                                {
                                    best_win_chance = p;
                                }
                            }
                        }
                    }

                    // mark square with best win chance
                    if exists_safe_and_useful == false ||
                        mode == SearchMode::WinEight ||
                        mode == SearchMode::SurviveLevel ||
                        mode == SearchMode::SurviveEight
                    {
                        for r in 0..5
                        {
                            for c in 0..5
                            {
                                if let Some(p) = probs[r][c]
                                {
                                    square_buttons[r][c].set_color(Color::color_average(Color::Cyan, Color::White, 0.3));
                                    square_buttons[r][c].set_label_size(half_button_size / 2);
                                }
                            }
                        }
                    }

                    if mode != SearchMode::Coins
                    {
                        square_buttons[row][col].set_label(&format!("{:.0}%", val * 100.0));
                    }
                    else
                    {
                        square_buttons[row][col].set_label(&format!("{:.0}", val));
                    }
                }

                ReportMessage::SquareSymbols(sp) => {
                    info!("GUI: Received symbol probs for all squares");
                    symbol_probs = Some(sp);
                    window.set_label("Calculating...");

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

                                if mode != SearchMode::WinEight &&
                                    mode != SearchMode::SurviveEight &&
                                    mode != SearchMode::SurviveLevel
                                {
                                    square_buttons[r][c].set_color(color);
                                }
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
                    info!("GUI: Received redundant stop confirmation");
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

                    info!("GUI: Square ({}, {}) was updated to {}", r, c, state[r][c]);

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

                    info!("GUI: Level was updated to {}", level);

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
                    tell_thread_start(&to_thread);
                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                },

                Reset => {

                    info!("GUI: Reset state, level and constraints");

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

                    info!("GUI: Sum of points of row {} updated to {}", r, sr[r]);

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

                    info!("GUI: Sum of points of col {} updated to {}", c, sc[c]);

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

                    info!("GUI: Bomb count of row {} updated to {}", r, br[r]);

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

                    info!("GUI: Bomb count of col {} updated to {}", c, bc[c]);

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_constraints(&to_thread, &sr, &sc, &br, &bc, level);
                    tell_thread_start(&to_thread);
                    window.set_label(TITLE_CALCULATING_POSSIBLE_BOARDS);
                }

                Mode(m) => {

                    if m != mode
                    {
                        info!("GUI: Changing mode to {:?}", m);
                    }

                    text_display.buffer().unwrap().set_text({
                        match m
                        {
                            SearchMode::WinChance => DESCRIPTION_WIN,
                            SearchMode::WinEight => DESCRIPTION_WIN_EIGHT,
                            SearchMode::SurviveNextMove => DESCRIPTION_SURVIVE_NEXT_MOVE,
                            SearchMode::SurviveLevel => DESCRIPTION_SURVIVE_LEVEL,
                            SearchMode::SurviveEight => DESCRIPTION_SURVIVE_EIGHT,
                            SearchMode::Coins => DESCRIPTION_COINS,
                        }
                    });
                    mode = m;

                    tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                    tell_thread_mode(&to_thread, mode);
                    tell_thread_start(&to_thread);
                }

                Threads => {
                    let v = counter.value() as usize;

                    if threads != v
                    {
                        threads = v;

                        info!("GUI: Set number of threads to {}", threads);

                        tell_thread_to_stop_and_wait_till_it_is_stopped(&to_thread, &from_thread);
                        tell_thread_threads(&to_thread, threads);
                        tell_thread_start(&to_thread);
                    }
                }
            }

            Tooltip::disable();
            symbol_probs = None;
            probs = [[None;5];5];

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