use ncurses::*;

use flow::line::Line;
use ui::menu::Menu;
use ui::content::Content;
use ui::printer::Print;
use ui::color;

static MAX_SCROLLING_LINES: i32 = 10_000;

pub enum Direction {
    Left,
    Right
}

pub enum Event {
    SelectMenuItem(Direction),
    ScrollContents(i32),
    Resize,
    Other
}

pub struct Ui {
    pub screen_lines: i32,
    menu: Menu,
    pub content: Content,
    height: i32,
    width: i32
}

impl Ui {
    pub fn new(menu_items: &Vec<String>) -> Ui {
        setlocale(LcCategory::all, ""); // Must be set *before* init

        initscr();
        start_color();
        use_default_colors();
        cbreak();
        noecho();
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        halfdelay(1);
        mouseinterval(0);
        keypad(stdscr, true);

        init_pair(1, COLOR_WHITE, COLOR_BLUE);
        init_pair(2, COLOR_WHITE, COLOR_GREEN);
        color::generate_pairs();

        Ui {
            menu: Menu::new(LINES - 1, 0, menu_items),
            content: Content::new(MAX_SCROLLING_LINES, COLS),
            screen_lines: 0,
            height: LINES,
            width: COLS
        }
    }

    pub fn render(&self) {
        self.menu.render(COLOR_PAIR(1), COLOR_PAIR(2));
        self.content.render();
    }

    pub fn select_left_menu_item(&self) {
        self.menu.select(REQ_LEFT_ITEM);
    }

    pub fn select_right_menu_item(&self) {
        self.menu.select(REQ_RIGHT_ITEM);
    }

    pub fn destroy(&self) {
        self.menu.destroy();
        endwin();
    }

    pub fn resize(&mut self) {
        getmaxyx(stdscr, &mut self.height, &mut self.width);
        self.content.resize(MAX_SCROLLING_LINES, self.width);
        mvwin(self.menu.window, self.height - 1, 0);
        refresh();
        wrefresh(self.menu.window);
    }

    pub fn print<'a>(&mut self, data: (Box<Iterator<Item=&'a Line> + 'a>, usize)) {
        let (lines, scroll_offset) = data;

        for line in lines {
            line.print(&self.content);
        }

        self.screen_lines = self.content.height();
        self.scroll(scroll_offset as i32);
    }

    pub fn scroll(&self, reversed_offset: i32) {
        let offset =  self.screen_lines - self.height + 1 - reversed_offset;
        prefresh(self.content.window, offset, 0, 0, 0, self.height - 2, self.width);
    }

    pub fn watch(&self) -> Event {
        match getch() {
            KEY_LEFT   => Event::SelectMenuItem(Direction::Left),
            KEY_RIGHT  => Event::SelectMenuItem(Direction::Right),
            KEY_UP     => Event::ScrollContents(1),
            KEY_DOWN   => Event::ScrollContents(-1),
            KEY_MOUSE  => self.read_mouse_event(),
            KEY_RESIZE => Event::Resize,
            _ => Event::Other
        }
    }

    fn read_mouse_event(&self) -> Event {
        let ref mut event = MEVENT {
            id: 0, x: 0, y: 0, z: 0, bstate: 0
        };
        if getmouse(event) == OK {
            if (event.bstate & BUTTON4_PRESSED as u64) != 0 {
                return Event::ScrollContents(1)
            } else if (event.bstate & BUTTON5_PRESSED as u64) != 0 {
                return Event::ScrollContents(-1)
            }
        }
        Event::Other
    }
}