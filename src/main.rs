use ncurses::{self as nc};
use rmatrix::Matrix;

const GREEN: i16 = 1;
const LIGHT_GREEN: i16 = 2;

fn main() {
    let root = nc::initscr();
    nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    nc::start_color();
    nc::init_pair(GREEN, nc::COLOR_GREEN, nc::COLOR_BLACK);
    nc::init_pair(LIGHT_GREEN, nc::COLOR_WHITE, nc::COLOR_BLACK);
    Matrix::new(root).run();
    nc::endwin();
}
