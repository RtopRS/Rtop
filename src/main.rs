use rtop_rs::*;
use ncurses::*;
use std::sync::Arc;
use chrono::Timelike;
use sysinfo::{ProcessExt, SystemExt, ProcessorExt};


#[tokio::main]
async fn main() {
    let locale = setlocale(LcCategory::all, "");
    if !locale.contains("UTF-8") {
        println!("You need to suppoort UTF-8");
        return;
    }

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
                curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_VISIBLE);
                print!("\x1b[?25h");
                std::process::exit(0);
            },
            Err(_) => {}
          }
    });

    let term = initscr();
    keypad(term, true);
    start_color();
    use_default_colors();

    init_pair(1, COLOR_GREEN, -1);
    init_pair(2, COLOR_BLUE, -1);

    match curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE) {
        None => {
            print!("\x1b[?25l");
        }
        Some(_) => ()
    }

    let mut height = 0;
    let mut width = 0;

    getmaxyx(term, &mut height, &mut width);

    attron(ncurses::A_BOLD());
    attron(COLOR_PAIR(2));
    addstr(" rtop ");
    attrset(ncurses::A_NORMAL());
    addstr(&format!("for {}", current_os));
    refresh();

    display_help(height);

    height -= 2;

    let mut cpu_win = window::Window::new(height / 2, width, 0, 1, COLOR_PAIR(1), COLOR_PAIR(2), "CPU Usage".to_string());
    let mut memory_win = window::Window::new(height - cpu_win.height, width / 2, 0, cpu_win.height + 1, COLOR_PAIR(1), COLOR_PAIR(2), "Memory Usage".to_string());
    let mut process_win = window::Window::new(height - cpu_win.height, width - memory_win.width, memory_win.width, cpu_win.height + 1, COLOR_PAIR(1), COLOR_PAIR(2), "Process List".to_string());
    
    let mut chart = widget::chart::Chart::new(memory_win.width - 2, memory_win.height - 2, true);
    let mut cpu_chart = widget::chart::Chart::new(cpu_win.width - 2, cpu_win.height - 2, true);
    let mut process_list = widget::listview::ListView::new(process_win.height - 2, process_win.width - 2, &*processes_list.lock().await, "Name", vec!("CPU %".to_string(), "Count".to_string(), "Memory %".to_string()));

    timeout(334);
    noecho();

    let mut name_to_find_key_kill_process = false;

    loop {
        match getch() {
            ncurses::KEY_DOWN => { process_list.next() },
            ncurses::KEY_UP => { process_list.previous() },
            ncurses::KEY_RESIZE => {
                erase();
                resize_term(0, 0);
                refresh();
                getmaxyx(term, &mut height, &mut width);
                resizeterm(0, 0);
                attron(ncurses::A_BOLD());
                attron(COLOR_PAIR(2));
                addstr(" rtop ");
                attrset(ncurses::A_NORMAL());
                addstr(&format!("for {}", current_os));
                display_help(height);

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
            100 => {
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
            113 => { break }
            103 => { process_list.to_first() }
            71 => { process_list.to_last() }
            109 => { process_list.sort_by("Memory %") }
            99 => { process_list.sort_by("CPU %") }
            67 => { process_list.sort_by("Count") }
            _ => name_to_find_key_kill_process = false
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
        mvaddstr(0, width - 9, &format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second()));
        let load_average = load_avg_data.lock().await;
        mvaddstr(0, width / 2 - (load_average.len() / 2) as i32, &*load_average);
    }

    endwin();
    print!("\x1b[?25h");
    std::process::exit(0);
}

fn display_help(win_height: i32) {
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
    
    mv(win_height - 1, 0);

    for (key, value) in help {
        attron(ncurses::A_BOLD());
        attron(COLOR_PAIR(2));
        addstr(&format!(" {} ", key));
        attroff(ncurses::A_BOLD());
        attroff(COLOR_PAIR(2));
        addstr(&format!("{} ", value));
    }
}