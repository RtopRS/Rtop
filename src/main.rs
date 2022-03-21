extern crate pancurses;
extern crate systemstat;

use pancurses::*;
use chrono::Timelike;

mod window;
mod chart;

use std::sync::Arc;
use crate::systemstat::Platform;

fn get_linux_distribution() -> String {
    if std::path::Path::new("/etc/os-release").exists(){
        let contents = std::fs::read_to_string("/etc/os-release").unwrap();
        return contents.split("NAME=").collect::<Vec<&str>>()[1].split("\n").collect::<Vec<&str>>()[0].to_string().replace("\"", "").replace(" ", "");
    }
    "".to_string()
}

#[tokio::main]
async fn main() {
    let sys = systemstat::System::new();
    let memory_data = Arc::new(tokio::sync::Mutex::new(vec!()));
    let memory_mutex = Arc::clone(&memory_data);
    let cpu_data = Arc::new(tokio::sync::Mutex::new(vec!()));
    let cpu_mutex = Arc::clone(&cpu_data);
    let load_avg_data = Arc::new(tokio::sync::Mutex::new("".to_string()));
    let load_avg_mutex = Arc::clone(&load_avg_data);

    tokio::spawn(async move {
        let mut last_cpu = 0.;
        loop {
            match sys.cpu_load_aggregate() {
                Ok(cpu) => {
                    std::thread::sleep(std::time::Duration::from_millis(333));
                    let cpu = cpu.done().unwrap();
                    let mut cpu_lock = cpu_mutex.lock().await;
                    cpu_lock.push(((cpu.user + last_cpu) / 2. * 100.) as i32);
                    last_cpu = cpu.user;
                }
                Err(_) => ()
            };

            match sys.load_average() {
                Ok(loadavg) => {
                    let mut load_average = load_avg_mutex.lock().await;
                    *load_average = format!(" Load Average: {:.2} {:.2} {:.2} ", loadavg.one, loadavg.five, loadavg.fifteen);
                },
                Err(_) => ()
            }

            let mem = sys.memory().unwrap();
            let mut mem_lock = memory_mutex.lock().await;
            mem_lock.push(((mem.total.as_u64() - mem.free.as_u64()) * 100 / mem.total.as_u64()) as i32);
        }
    });

    let current_os = get_linux_distribution();

    tokio::spawn(async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                endwin();
                print!("\x1b[?25h");
                std::process::exit(0);
            },
            Err(_) => {}
          }
    });

    let mut term = initscr();
    term.keypad(true);
    start_color();
    use_default_colors();

    init_pair(1, COLOR_GREEN, -1);
    init_pair(2, COLOR_BLUE, -1);

    let result = curs_set(0);
    if result == -1 {
        println!("\x1b[?25l");
    }

    let (mut height, mut width) = term.get_max_yx();

    term.attron(pancurses::A_BOLD);
    term.attron(ColorPair(2));
    term.addstr(" rtop ");
    term.attrset(pancurses::A_NORMAL);
    term.addstr(format!("for {}", current_os));
    term.attron(pancurses::A_BOLD);
    term.mvaddstr(height - 1, 1, "Q ");
    term.attroff(pancurses::A_BOLD);
    term.addstr("quit");
    term.refresh();

    height -= 2;

    let mut cpu_win = window::Window::new(height / 2, width, 0, 1, ColorPair(1), ColorPair(2), "CPU Usage".to_string());
    let mut memory_win = window::Window::new(height - cpu_win.height, width / 2, 0, cpu_win.height + 1, ColorPair(1), ColorPair(2), "Memory Usage".to_string());
    let mut process_win = window::Window::new(height - cpu_win.height, width - memory_win.width, memory_win.width, cpu_win.height + 1,  ColorPair(1), ColorPair(2), "Process List".to_string());
    
    let mut chart = chart::Chart::new(memory_win.width - 2, memory_win.height - 2, true);
    let mut cpu_chart = chart::Chart::new(cpu_win.width - 2, cpu_win.height - 2, true);

    let mut counter = 0;

    term.timeout(333);
    noecho();

    loop {
        match term.getch() {
            Some(pancurses::Input::KeyDown) => (counter -= 1),
            Some(pancurses::Input::KeyUp) => (counter += 1),
            Some(pancurses::Input::KeyResize) => {
                term.erase();
                term.resize(0, 0);
                (height, width) = term.get_max_yx();
                term.attron(pancurses::A_BOLD);
                term.attron(ColorPair(2));
                term.addstr(" rtop ");
                term.attrset(pancurses::A_NORMAL);
                term.addstr("for archlinux");
                term.mvaddstr(height - 1, 1, "Q ");
                term.attroff(pancurses::A_BOLD);
                term.addstr("quit");

                height -= 2;
                cpu_win.resize(height / 2, width);
                cpu_win.deplace(0, 1);
                memory_win.resize(height - cpu_win.height, width / 2);
                memory_win.deplace(0, cpu_win.height + 1);
                process_win.resize(height - cpu_win.height, width - memory_win.width);
                process_win.deplace(memory_win.width, cpu_win.height + 1);
                chart.resize(memory_win.width - 2, memory_win.height - 2);
                cpu_chart.resize(cpu_win.width - 2, cpu_win.height - 2);
            },
            Some(pancurses::Input::Character('q')) => { break }
            Some(_) => (),
            None => ()
        }
        memory_win.write(&chart.display(&*memory_data.lock().await).to_string());
        cpu_win.write(&cpu_chart.display(&*cpu_data.lock().await).to_string());
        process_win.write(&format!("{}", counter));
        process_win.refresh();
        cpu_win.refresh();
        memory_win.refresh();
        let now = chrono::Utc::now();
        term.mvaddstr(0, width - 9, &format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second()));
        let load_average = load_avg_data.lock().await;
        term.mvaddstr(0, width / 2 - (load_average.len() / 2) as i32, &*load_average);
        term.refresh();
    }

    endwin();
    print!("\x1b[?25h");
    std::process::exit(0);
}