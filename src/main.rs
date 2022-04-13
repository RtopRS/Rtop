use rtop_rs::*;
use pancurses::*;
use std::sync::Arc;
use chrono::Timelike;
use sysinfo::{ProcessExt, SystemExt, ProcessorExt};


#[tokio::main]
async fn main() {
    let mut sys_process_info = sysinfo::System::new_all();
    let mut sys_memory_info = sysinfo::System::new_all();
    let memory_data = Arc::new(tokio::sync::Mutex::new(vec!()));
    let memory_mutex = Arc::clone(&memory_data);
    let cpu_data = Arc::new(tokio::sync::Mutex::new(vec!()));
    let cpu_mutex = Arc::clone(&cpu_data);
    let load_avg_data = Arc::new(tokio::sync::Mutex::new("".to_string()));
    let load_avg_mutex = Arc::clone(&load_avg_data);
    let processes_list = Arc::new(tokio::sync::Mutex::new(vec!()));
    let processes_list_mutex = Arc::clone(&processes_list);

    let physical_core_count = sys_process_info.physical_core_count().unwrap();
    let current_os = sys_process_info.name().unwrap();

    tokio::spawn(async move {
        let mut last_cpu = 0.;
        loop {
            sys_memory_info.refresh_cpu();
            sys_memory_info.refresh_memory();
            let cpu_usage = sys_memory_info.global_processor_info().cpu_usage();
            {
                let mut cpu_lock = cpu_mutex.lock().await;
                cpu_lock.push(((cpu_usage + last_cpu) / 2.) as i32);
            }
            last_cpu = cpu_usage;


            {
                let load_average = sys_memory_info.load_average();
                let mut load_average_string = load_avg_mutex.lock().await;
                *load_average_string = format!(" Load Average: {:.2} {:.2} {:.2} ", load_average.one, load_average.five, load_average.fifteen);
            }
            {
                let mut mem_lock = memory_mutex.lock().await;
                mem_lock.push((sys_memory_info.used_memory() * 100 / sys_memory_info.total_memory()) as i32);
            }
            std::thread::sleep(std::time::Duration::from_millis(334));
        }
    });

    tokio::spawn(async move {
        loop {
            let mut process_done = vec!();
            let mut new_process_list = vec!();
            for (_, process) in sys_process_info.processes() {
                if process_done.contains(&process.name()) {
                    continue;
                }
                process_done.push(process.name());

                let mut process_data = std::collections::HashMap::new();
                let mut total_cpu = 0.;
                let mut total_memory: u64 = 0;
                let mut count: i32 = 0;
                for sub_proc in sys_process_info.processes_by_exact_name(process.name()) {
                    count += 1;
                    total_memory += sub_proc.memory();
                    total_cpu += sub_proc.cpu_usage()
                }
                process_data.insert("CPU %".to_string(), format!("{:.1}", (total_cpu / physical_core_count as f32)));
                process_data.insert("Count".to_string(), count.to_string());
                process_data.insert("Memory %".to_string(), format!("{:.1}", (total_memory as f32 * 100. / sys_process_info.total_memory() as f32)));

                new_process_list.push(widget::listview::ListItem::new(process.name(), &process_data));
            }
            {
                let mut process_list = processes_list_mutex.lock().await;
                *process_list = new_process_list
            }
            std::thread::sleep(std::time::Duration::from_millis(2500));
            sys_process_info.refresh_processes();
        }
    });

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
    display_help(&term, height);
    term.refresh();

    height -= 2;

    let mut cpu_win = window::Window::new(height / 2, width, 0, 1, ColorPair(1), ColorPair(2), "CPU Usage".to_string());
    let mut memory_win = window::Window::new(height - cpu_win.height, width / 2, 0, cpu_win.height + 1, ColorPair(1), ColorPair(2), "Memory Usage".to_string());
    let mut process_win = window::Window::new(height - cpu_win.height, width - memory_win.width, memory_win.width, cpu_win.height + 1,  ColorPair(1), ColorPair(2), "Process List".to_string());
    
    let mut chart = widget::chart::Chart::new(memory_win.width - 2, memory_win.height - 2, true);
    let mut cpu_chart = widget::chart::Chart::new(cpu_win.width - 2, cpu_win.height - 2, true);
    let mut process_list = widget::listview::ListView::new(process_win.height - 2, process_win.width - 2, &*processes_list.lock().await, "Name", vec!("CPU %".to_string(), "Count".to_string(), "Memory %".to_string()));

    term.timeout(334);
    noecho();

    let mut name_to_find_key_kill_process = false;

    loop {
        match term.getch() {
            Some(pancurses::Input::KeyDown) => { process_list.next() },
            Some(pancurses::Input::KeyUp) => { process_list.previous() },
            Some(pancurses::Input::KeyResize) => {
                term.erase();
                term.resize(0, 0);
                (height, width) = term.get_max_yx();
                term.attron(pancurses::A_BOLD);
                term.attron(ColorPair(2));
                term.addstr(" rtop ");
                term.attrset(pancurses::A_NORMAL);
                term.addstr(format!("for {}", current_os));
                display_help(&term, height);

                height -= 2;
                cpu_win.resize(height / 2, width);
                cpu_win.deplace(0, 1);
                memory_win.resize(height - cpu_win.height, width / 2);
                memory_win.deplace(0, cpu_win.height + 1);
                process_win.resize(height - cpu_win.height, width - memory_win.width);
                process_win.deplace(memory_win.width, cpu_win.height + 1);
                chart.resize(memory_win.width - 2, memory_win.height - 2);
                cpu_chart.resize(cpu_win.width - 2, cpu_win.height - 2);
                process_list.resize(process_win.height - 2, process_win.width - 2);
            },
            Some(pancurses::Input::Character('d')) => {
                if name_to_find_key_kill_process {
                    process_list.select(|item| {
                        let sys_process_info = sysinfo::System::new_all();
                        for proc in sys_process_info.processes_by_exact_name(&item.name) {
                            proc.kill();
                        }
                    });
                }
                name_to_find_key_kill_process = !name_to_find_key_kill_process;
            }
            Some(pancurses::Input::Character('q')) => { break }
            Some(pancurses::Input::Character('g')) => { process_list.to_first() }
            Some(pancurses::Input::Character('G')) => { process_list.to_last() }
            Some(pancurses::Input::Character('m')) => { process_list.sort_by("Memory %") }
            Some(pancurses::Input::Character('c')) => { process_list.sort_by("CPU %") }
            Some(pancurses::Input::Character('C')) => { process_list.sort_by("Count") }
            Some(pancurses::Input::Character('n')) => { process_list.sort_by("Name") }
            Some(_) => (),
            None => { name_to_find_key_kill_process = false }
        }
        {
            process_list.update_items(&*processes_list.lock().await);
            memory_win.write(&chart.display(&*memory_data.lock().await).to_string());
            cpu_win.write(&cpu_chart.display(&*cpu_data.lock().await).to_string());
        }
        process_win.write(&format!("{}", process_list.display()));
        process_win.refresh();
        cpu_win.refresh();
        memory_win.refresh();
        let now = chrono::Local::now();
        term.mvaddstr(0, width - 9, &format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second()));
        let load_average = load_avg_data.lock().await;
        term.mvaddstr(0, width / 2 - (load_average.len() / 2) as i32, &*load_average);
    }

    endwin();
    print!("\x1b[?25h");
    std::process::exit(0);
}

fn display_help(term: &Window, win_height: i32) {
    let mut help: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    help.insert("Q", "Quit");
    help.insert("J", "Down");
    help.insert("K", "Up");
    help.insert("g", "Jump to top");
    help.insert("G", "Jump to bottom");
    help.insert("dd", "Kill process");
    help.insert("m", "Sort by memory");
    help.insert("n", "Sort by name");
    help.insert("c", "Sort by CPU");
    help.insert("C", "Sort by count");
    
    term.mv(win_height - 1, 0);

    for (key, value) in help {
        term.attron(pancurses::A_BOLD);
        term.attron(pancurses::ColorPair(2));
        term.addstr(format!(" {} ", key));
        term.attroff(pancurses::A_BOLD);
        term.attroff(pancurses::ColorPair(2));
        term.addstr(format!("{} ", value));
    }
}