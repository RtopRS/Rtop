pub trait Plugin {
    fn display(&mut self, height: i32, width: i32) -> String;

    fn update(&mut self); // called every time the loop for ac tualize data is done

    fn refresh_rate(&self) -> i32 { // set the time between two refresh loop execution
        333
    }

    fn resize(&mut self, _h: i32, _w: i32) {
        
    }

    fn on_input(&mut self, _key: String) {

    }
}