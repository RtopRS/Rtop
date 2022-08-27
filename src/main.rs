use rtop_dev::*;
use ncurses::*;
use chrono::Timelike;
use sysinfo::{ProcessExt, SystemExt, ProcessorExt};
use serde::Deserialize;
use rtop_rs::window;


fn default_pages() -> Vec<Vec<String>> {
    vec!(vec!("cpu_chart".to_string(), "memory_chart".to_string(), "process_list".to_string()))
}
#[derive(Deserialize)]
struct Option {
    #[serde(default = "default_pages")]
    pages: Vec<Vec<String>>,
    #[serde(default)]
    plugins: Vec<LibOption>
}
#[derive(Deserialize)]
struct LibOption {
    #[serde(default)]
    path: String,
    #[serde(default)]
    provided_widgets: Vec<String>
}

struct MemoryUsage {
    sysinfo: sysinfo::System,
    data: Vec<i32>,
    chart: widget::chart::Chart
}
struct PluginError {
    message: String
}
struct CpuUsage {
    sysinfo: sysinfo::System,
    data: Vec<i32>,
    chart: widget::chart::Chart,
    last_cpu_usage: f32
}
struct ProcessList {
    sysinfo: sysinfo::System,
    data: Vec<widget::listview::ListItem>,
    chart: widget::listview::ListView,
    refresh_progress: usize,
    kill_process_security: bool
}

struct Test {}
impl plugin::Plugin for Test {
    fn display(&mut self, _height: i32, _width: i32) -> String {
        String::from("common text [[EFFECT_COLOR_GREEN]]green text [[EFFECT_REVERSE]]and he is inverted here [[EFFECT_COLOR_RED]] with red color [[EFFECT_COLOR_RED]][[EFFECT_REVERSE]] and re common text")
    }
}


impl plugin::Plugin for ProcessList {
    fn on_update(&mut self) {
        self.refresh_progress += 1;
        if self.refresh_progress == 7 {
            let mut process_done = vec!();
            let mut new_process_list = vec!();
            let physical_core_count = self.sysinfo.physical_core_count().unwrap();

            for process in self.sysinfo.processes().values() {
                if process_done.contains(&process.name()) {
                    continue;
                }
                process_done.push(process.name());

                let mut process_data = std::collections::HashMap::new();
                let mut total_cpu = 0.;
                let mut total_memory: u64 = 0;
                let mut count: i32 = 0;
                for sub_proc in self.sysinfo.processes_by_exact_name(process.name()) {
                    count += 1;
                    total_memory += sub_proc.memory();
                    total_cpu += sub_proc.cpu_usage()
                }
                process_data.insert("CPU %".to_string(), format!("{:.1}", (total_cpu / physical_core_count as f32)));
                process_data.insert("Count".to_string(), count.to_string());
                process_data.insert("Memory %".to_string(), format!("{:.1}", (total_memory as f32 * 100. / self.sysinfo.total_memory() as f32)));

                new_process_list.push(widget::listview::ListItem::new(process.name(), &process_data));
            }

            self.data = new_process_list;
            self.sysinfo.refresh_processes();
            self.refresh_progress = 0;
        }
    }

    fn display(&mut self, h: i32, w: i32) -> String {
        self.chart.resize(h, w);
        self.chart.update_items(&self.data);
        self.chart.display()
    }

    fn on_input(&mut self, key: String) {
        if key == "KEY_DOWN" || key == "k" {
            self.chart.next();
        } else if key == "KEY_UP" || key == "j" {
            self.chart.previous();
        } else if key == "g" {
            self.chart.to_first();
        } else if key == "G" {
            self.chart.to_last();
        } else if key == "m" {
            self.chart.sort_by("Memory %");
        } else if key == "c" {
            self.chart.sort_by("CPU %");
        } else if key == "n" {
            self.chart.sort_by("Name");
        } else if key == "C" {
            self.chart.sort_by("Count");
        } else if key == "d" {
            if self.kill_process_security {
                let item = self.chart.select();
                for proc in sysinfo::System::new_all().processes_by_exact_name(&item.name) { //replace this by self.sysinfo
                    proc.kill();
                }
            }
            self.kill_process_security = !self.kill_process_security;
        }
        if key != "d" && self.kill_process_security {
            self.kill_process_security = false;
        }
    }

    fn title(&mut self) -> String {
        String::from("Processes")
    }
}
impl plugin::Plugin for CpuUsage {
    fn on_update(&mut self) {
        self.sysinfo.refresh_cpu();
        self.data.push(((self.sysinfo.global_processor_info().cpu_usage() + self.last_cpu_usage) / 2.) as i32)
    }

    fn display(&mut self, h: i32, w: i32) -> String {
        self.chart.resize(w, h);
        self.chart.display(&self.data)
    }

    fn title(&mut self) -> String {
        String::from("CPU Usage")
    }
}
impl plugin::Plugin for MemoryUsage {
    fn display(&mut self, h: i32, w: i32) -> String {
        self.chart.resize(w, h);
        self.chart.display(&self.data)
    }

    fn on_update(&mut self) {
        self.sysinfo.refresh_memory();
        self.data.push((self.sysinfo.used_memory() * 100 / self.sysinfo.total_memory()) as i32);
    }

    fn title(&mut self) -> String {
        String::from("Memory")
    }
}
impl plugin::Plugin for PluginError {
    fn display(&mut self, h: i32, w: i32) -> String {
        let error_message = format!("An error occured: {}", self.message);
        format!("{}{}{}", String::from("\n").repeat((h / 2) as usize), String::from(" ").repeat((w / 2) as usize - error_message.len() / 2) ,error_message)
    }
}

struct ScreenWidget {
    plugin: Box<dyn plugin::Plugin>,
    name: String
}
struct Page {
    widgets: Vec<ScreenWidget>,
    focusable_widgets: Vec<i32>
}
unsafe impl Send for Page {}


#[tokio::main]
async fn main() {
    let option: Option = serde_json::from_str(&std::fs::read_to_string(format!("{}/.config/rtop/config", home::home_dir().unwrap().display())).unwrap_or_else(|_| "{}".to_string())).unwrap();

    // loads all dynamic libs
    let mut libs = std::collections::HashMap::new();
    let mut extern_addon = std::collections::HashMap::new();
    for lib in option.plugins {
        unsafe {
            let loaded_lib = libloading::Library::new(String::from(&lib.path));
            if loaded_lib.is_ok() {
                libs.insert(String::from(&lib.path), loaded_lib.unwrap());
            } else {
                continue;
            }
        }
        for widget in lib.provided_widgets {
            extern_addon.insert(widget, String::from(&lib.path));
        }
    }

    let mut builtin_addon: std::collections::HashMap<String, fn() -> (Box<dyn plugin::Plugin>, bool)> = std::collections::HashMap::new();
    builtin_addon.insert("memory_chart".to_string(), init_memory_plugin);
    builtin_addon.insert("cpu_chart".to_string(), init_cpuusage_plugin);
    builtin_addon.insert("process_list".to_string(), init_process_plugin);

    builtin_addon.insert("test".to_string(), init_test_plugin);

    // New Variable for Plugin Rework
    let mut current_page_number = 1;
    let sysinfo = sysinfo::System::new_all();
    let mut current_widget = 1;

    let locale = setlocale(LcCategory::all, "");
    if !locale.contains("UTF-8") {
        println!("You need to suppoort UTF-8");
        return;
    }

    //Initialize Ncurses
    let term = initscr();
    keypad(term, true);
    start_color();
    use_default_colors();
    let mut height = 0;
    let mut width = 0;
    getmaxyx(term, &mut height, &mut width);
    curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    timeout(334);
    noecho();

    init_pair(1, COLOR_RED, -1);
    init_pair(2, COLOR_GREEN, -1);
    init_pair(3, COLOR_YELLOW, -1);
    init_pair(4, COLOR_BLUE, -1);
    init_pair(5, COLOR_MAGENTA, -1);
    init_pair(6, COLOR_CYAN, -1);
    init_pair(7, COLOR_WHITE, -1);
    init_pair(8, COLOR_BLACK, -1);

    let pages: std::sync::Arc<tokio::sync::Mutex<std::vec::Vec<Page>>> = std::sync::Arc::new(tokio::sync::Mutex::new(vec!()));
    let pages_mutex = std::sync::Arc::clone(&pages);

    for page in option.pages {
        if page.len() > 4 {
            pages.lock().await.push(Page{widgets: vec!(ScreenWidget{name: String::from("Error"), plugin: Box::new(PluginError{message: String::from("You cannot have more than 4 widgets per pages")})}), focusable_widgets: vec!()})
        } else if page.is_empty() {
            pages.lock().await.push(Page{widgets: vec!(ScreenWidget{name: String::from("Error"), plugin: Box::new(PluginError{message: String::from("You must add a widget to this page")})}), focusable_widgets: vec!()})
        } else {
            let mut i = 0;
            let mut pages_widgets = vec!();
            let mut focusable_widgets = vec!();

            for widget in page {
                i += 1;
                if builtin_addon.contains_key(&widget) {
                    let tmp = builtin_addon[&widget]();
                    if tmp.1 {
                        focusable_widgets.push(i);
                    }
                    pages_widgets.push(ScreenWidget{name: String::from(&widget), plugin: tmp.0 });
                } else if extern_addon.contains_key(&widget) {
                    let addon_path = &extern_addon[&widget];
                    let lib = &libs[&String::from(addon_path)];
                    let tmp_lib = unsafe { lib.get(format!("init_{}", widget).as_bytes()) };
                    let initializer: libloading::Symbol<extern "Rust" fn() -> (Box<dyn plugin::Plugin>, bool)>;
                    if tmp_lib.is_ok() {
                        initializer = tmp_lib.unwrap();
                        let tmp = initializer();
                        if tmp.1 {
                            focusable_widgets.push(i);
                        }
                        pages_widgets.push(ScreenWidget{name: widget, plugin: tmp.0})
                    } else {
                        pages_widgets.push(ScreenWidget{name: String::from("Error"), plugin: Box::new(PluginError{message: String::from(format!("Widget {} cannot be loaded. Maybe the plugin is outdated?", widget, ))})})
                    }
                    
                } else {
                    pages_widgets.push(ScreenWidget{name: String::from("Error"), plugin: Box::new(PluginError{message: String::from(format!("Unknow widget {}", widget))})});
                }
            }
            
            pages.lock().await.push(Page{widgets: pages_widgets, focusable_widgets})
                
        }
    }

    tokio::spawn(async move {
        loop {
            for page in &mut *pages_mutex.lock().await {
                for el in &mut page.widgets {
                    el.plugin.on_update()
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(333));
        }
    });
    
    let mut widgets;
    {
        let locked_pages = pages.lock().await;
        widgets = create_widget_window(height - 2, width, locked_pages[current_page_number - 1].widgets.len() as i32);
    }

    let current_os = sysinfo.name().unwrap();
    attron(ncurses::A_BOLD());
    attron(COLOR_PAIR(4));
    addstr(" rtop ");
    attrset(ncurses::A_NORMAL());
    addstr(&format!("for {}", current_os));
    refresh();
    display_help(height);

    loop {
        let current_page_widget_count;
        let current_page_focusable_widget_count;
        {
            let locked_pages = &mut pages.lock().await;
            let current_page = &mut locked_pages[current_page_number - 1];
            current_page_widget_count = current_page.widgets.len();
            current_page_focusable_widget_count = current_page.focusable_widgets.len();

            if current_page_focusable_widget_count == 0 {
                current_widget = 0;
            }

            for i in 0..current_page_widget_count { // display all current page's widget
                let current_widget_window = &mut widgets[i];
                let widget = &mut current_page.widgets[i];
                current_widget_window.write(&widget.plugin.display(current_widget_window.height - 2, current_widget_window.width - 2));
                let mut title = &widget.plugin.title();
                if title == "" {
                    title = &widget.name;
                }
                current_widget_window.set_title(title.to_string());
                current_widget_window.set_border_color(COLOR_PAIR(2));
            }
            if current_page_focusable_widget_count > 1 {
                let tmp = current_page.focusable_widgets[current_widget - 1];
                widgets[(tmp as usize) - 1].set_border_color(COLOR_PAIR(4));
                widgets[(tmp as usize) - 1].refresh();
            }
            for i in 0..current_page_widget_count {
                widgets[i].refresh();
            }

        }
        
        // Update TopBar and BottomBar Infos
        let now = chrono::Local::now();
        let load_average = sysinfo.load_average();
        let load_average_string = format!(" Load Average: {:.2} {:.2} {:.2} ", load_average.one, load_average.five, load_average.fifteen);
        mvaddstr(0, width - 9, &format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second()));
        let page_indicator = format!("[{}/{}]", current_page_number, pages.lock().await.len());
        mvaddstr(height -1,  width - 1 - page_indicator.len() as i32, &page_indicator);
        mvaddstr(0, width / 2 - (load_average_string.len() / 2) as i32, &load_average_string);

        let key = getch();
        match key {
            ncurses::KEY_RIGHT => {
                let locked_pages = pages.lock().await;
                current_page_number += 1;
                if current_page_number > locked_pages.len() {
                    current_page_number = locked_pages.len();
                }

                widgets = create_widget_window(height - 2, width, locked_pages[current_page_number - 1].widgets.len() as i32);
                current_widget = 1;
            }
            ncurses::KEY_LEFT => {
                let locked_pages = pages.lock().await;
                current_page_number -= 1;
                if current_page_number < 1 {
                    current_page_number = 1;
                }
                widgets = create_widget_window(height - 2, width, locked_pages[current_page_number - 1].widgets.len() as i32);
                current_widget = 1;
            }
            ncurses::KEY_RESIZE => {
                erase();
                refresh();
                getmaxyx(term, &mut height, &mut width);
                resizeterm(0, 0);
                attron(ncurses::A_BOLD());
                attron(COLOR_PAIR(4));
                addstr(" rtop ");
                attrset(ncurses::A_NORMAL());
                addstr(&format!("for {}", current_os));
                display_help(height);
                widgets = create_widget_window(height - 2, width, current_page_widget_count as i32);
            }
            9 => { // TAB Key
                current_widget += 1;
                if current_widget > current_page_focusable_widget_count {
                    current_widget = current_page_focusable_widget_count;
                }
            }
            353 => { // BackTAB Key
                current_widget -= 1;
                if current_widget < 1 {
                    current_widget = 1;
                }
            }
            90 => { // BackTAB Key
                current_widget -= 1;
                if current_widget < 1 {
                    current_widget = 1;
                }
            }
            113 => { exit() }
            _ => {
                if current_widget != 0 {
                    let locked_pages = &mut pages.lock().await;
                    let current_page = &mut locked_pages[current_page_number - 1];
                    current_page.widgets[(current_page.focusable_widgets[current_widget - 1] as usize) - 1].plugin.on_input(ncurses::keyname(key).unwrap());
                }
            }
        }
    }
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
        attron(COLOR_PAIR(4));
        addstr(&format!(" {} ", key));
        attroff(ncurses::A_BOLD());
        attroff(COLOR_PAIR(4));
        addstr(&format!("{} ", value));
    }
}

fn exit() {
    endwin();
    curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_VISIBLE);
    std::process::exit(0);
}

fn create_widget_window(height: i32, width: i32, widget_count: i32) -> Vec<window::Window> {
    let mut win_height = height;
    let mut win_width = width;
    if widget_count >= 2 {
        win_height = height / 2;
    }
    if widget_count == 4 {
        win_width = width / 2;
    }
    let mut widgets = vec!();
    let widget1;
    let mut widget2;
    let widget3;
    let widget4;
    widget1 = window::Window::new(win_height, win_width, 0, 1, COLOR_PAIR(2), COLOR_PAIR(4), String::from("1"));
    widget2 = window::Window::new(height - win_height, win_width, 0, 1 + win_height, COLOR_PAIR(2), COLOR_PAIR(4), String::from("2"));
    if widget_count == 3 {
        widget2 = window::Window::new(height - win_height, (width as f32 / 2.).ceil() as i32, 0, 1 + win_height, COLOR_PAIR(2), COLOR_PAIR(4), String::from("2"));
    } else if widget_count == 4 {
        widget2 = window::Window::new(win_height, width - win_width, win_width, 1, COLOR_PAIR(2), COLOR_PAIR(4), String::from("2"));
    }
    if widget_count == 4 {
        widget3 = window::Window::new(height - win_height, win_width, 0, 1 + win_height, COLOR_PAIR(2), COLOR_PAIR(4), String::from("3"));
    } else {
        widget3 = window::Window::new(height - win_height, width - (width as f32 / 2.).ceil() as i32, width - ((width / 2) as i32), 1 + win_height, COLOR_PAIR(2), COLOR_PAIR(4), String::from("3"));
    }
    widget4 = window::Window::new(height - win_height, width - win_width, win_width, 1 + win_height, COLOR_PAIR(2), COLOR_PAIR(4), String::from("4"));
    widgets.push(widget1);
    widgets.push(widget2);
    widgets.push(widget3);
    widgets.push(widget4);
    widgets
}


fn init_cpuusage_plugin() -> (Box<dyn plugin::Plugin>, bool ){
    (Box::new(CpuUsage{data: Vec::new(), chart: widget::chart::Chart{cols: 0, rows: 0, show_percent: true}, sysinfo: sysinfo::System::new_all(), last_cpu_usage: 0.}), false)
}
fn init_memory_plugin() -> (Box<dyn plugin::Plugin>, bool) {
    (Box::new(MemoryUsage{sysinfo: sysinfo::System::new_all(), data: vec!(), chart: widget::chart::Chart{cols: 0, rows: 0, show_percent: true}}), false)
}
fn init_process_plugin() -> (Box<dyn plugin::Plugin>, bool) {
    (Box::new(ProcessList{sysinfo: sysinfo::System::new_all(), data: vec!(), chart: widget::listview::ListView::new(0, 0, &Vec::new(), "Name", vec!("CPU %".to_string(), "Count".to_string(), "Memory %".to_string())), refresh_progress: 6, kill_process_security: false}), true)
}

fn init_test_plugin() -> (Box<dyn plugin::Plugin>, bool) {
    (Box::new(Test{}), true)
}