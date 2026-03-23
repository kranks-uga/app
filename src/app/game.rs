//! Мини-игры: крестики-нолики, змейка, тетрис, сапёр

use std::collections::VecDeque;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::PathBuf;
use super::constants::{CONFIG_APP_NAME, GAMES_DIR};

// ============================================================================
// Диспетчер игр
// ============================================================================

#[derive(PartialEq, Clone, Copy)]
pub enum ActiveGame { TicTacToe, Snake, Tetris, Minesweeper }

pub struct GameHub {
    pub active:      ActiveGame,
    pub tictactoe:   TicTacToe,
    pub snake:       SnakeGame,
    pub tetris:      TetrisGame,
    pub minesweeper: MinesweeperGame,
}

impl GameHub {
    pub fn new() -> Self {
        let mut hub = Self {
            active:      ActiveGame::TicTacToe,
            tictactoe:   TicTacToe::new(GameMode::VsAi),
            snake:       SnakeGame::new(),
            tetris:      TetrisGame::new(),
            minesweeper: MinesweeperGame::new(),
        };

        // Попробуем загрузить сохранённые счёты при старте
        if let Ok(data) = GameHub::read_game_json::<TicTacToeScore>("tictactoe_score.json") {
            hub.tictactoe.score_x = data.score_x;
            hub.tictactoe.score_o = data.score_o;
            hub.tictactoe.draws = data.draws;
        }
        if let Ok(data) = GameHub::read_game_json::<SnakeScore>("snake_score.json") {
            hub.snake.high_score = data.high_score;
            hub.snake.score = data.last_score;
        }
        if let Ok(data) = GameHub::read_game_json::<TetrisScore>("tetris_score.json") {
            hub.tetris.score = data.score;
            hub.tetris.level = data.level;
            hub.tetris.lines = data.lines;
        }
        if let Ok(data) = GameHub::read_game_json::<MinesResult>("minesweeper_result.json") {
            hub.minesweeper.elapsed = data.elapsed;
        }

        hub
    }

    /// Сохраняет текущую активную игру в слот `name` (файл ~/.config/<app>/games/<name>.json)
    pub fn save_slot(&self, name: &str) -> Result<(), String> {
        let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push(CONFIG_APP_NAME);
        dir.push(GAMES_DIR);
        if let Err(e) = fs::create_dir_all(&dir) {
            return Err(format!("Не удалось создать директорию сохранений: {}", e));
        }
        let path = dir.join(format!("{}.json", name));

        match self.active {
            ActiveGame::TicTacToe => {
                let s = TicTacToeSave::from(&self.tictactoe);
                let json = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
                fs::write(path, json).map_err(|e| e.to_string())
            }
            ActiveGame::Snake => {
                let s = SnakeSave::from(&self.snake);
                let json = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
                fs::write(path, json).map_err(|e| e.to_string())
            }
            ActiveGame::Tetris => {
                let s = TetrisSave::from(&self.tetris);
                let json = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
                fs::write(path, json).map_err(|e| e.to_string())
            }
            ActiveGame::Minesweeper => {
                let s = MinesSave::from(&self.minesweeper);
                let json = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
                fs::write(path, json).map_err(|e| e.to_string())
            }
        }
    }

    /// Загружает слот `name` в активную игру (перезаписывает состояние соответствующей игры)
    pub fn load_slot(&mut self, name: &str) -> Result<(), String> {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(CONFIG_APP_NAME);
        path.push(GAMES_DIR);
        path.push(format!("{}.json", name));

        let data = fs::read_to_string(&path).map_err(|e| format!("Не удалось прочитать файл: {}", e))?;

        match self.active {
            ActiveGame::TicTacToe => {
                let s: TicTacToeSave = serde_json::from_str(&data).map_err(|e| e.to_string())?;
                s.apply(&mut self.tictactoe);
                Ok(())
            }
            ActiveGame::Snake => {
                let s: SnakeSave = serde_json::from_str(&data).map_err(|e| e.to_string())?;
                s.apply(&mut self.snake);
                Ok(())
            }
            ActiveGame::Tetris => {
                let s: TetrisSave = serde_json::from_str(&data).map_err(|e| e.to_string())?;
                s.apply(&mut self.tetris);
                Ok(())
            }
            ActiveGame::Minesweeper => {
                let s: MinesSave = serde_json::from_str(&data).map_err(|e| e.to_string())?;
                s.apply(&mut self.minesweeper);
                Ok(())
            }
        }
    }

    /// Вспомогательная функция записи JSON-данных в папку `~/.config/<app>/games`
    fn write_game_json<T: Serialize>(file_name: &str, value: &T) -> Result<(), String> {
        let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push(CONFIG_APP_NAME);
        dir.push(GAMES_DIR);
        if let Err(e) = fs::create_dir_all(&dir) {
            return Err(format!("Не удалось создать директорию сохранений: {}", e));
        }
        let path = dir.join(file_name);
        let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    fn read_game_json<T: DeserializeOwned>(file_name: &str) -> Result<T, String> {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(CONFIG_APP_NAME);
        path.push(GAMES_DIR);
        path.push(file_name);
        let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    }
}

impl Drop for GameHub {
    fn drop(&mut self) {
        let _ = self.tictactoe.save_scores();
        let _ = self.snake.save_score();
        let _ = self.tetris.save_score();
        let _ = self.minesweeper.save_result();
    }
}

// ============================================================================
// Крестики-нолики
// ============================================================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell { Empty, X, O }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameMode { VsAi, TwoPlayers }

#[derive(Debug)]
pub enum GameState { Playing, Won(Cell), Draw }

pub struct TicTacToe {
    pub board:   [Cell; 9],
    pub current: Cell,
    pub state:   GameState,
    pub mode:    GameMode,
    pub score_x: u32,
    pub score_o: u32,
    pub draws:   u32,
}

impl TicTacToe {
    pub fn new(mode: GameMode) -> Self {
        Self { board: [Cell::Empty; 9], current: Cell::X, state: GameState::Playing,
               mode, score_x: 0, score_o: 0, draws: 0 }
    }
    pub fn reset(&mut self) {
        self.board = [Cell::Empty; 9];
        self.current = Cell::X;
        self.state = GameState::Playing;
    }
    pub fn make_move(&mut self, idx: usize) {
        if !matches!(self.state, GameState::Playing) || self.board[idx] != Cell::Empty { return; }
        self.board[idx] = self.current;
        if let Some(w) = ttt_winner(&self.board) {
            match w { Cell::X => self.score_x += 1, Cell::O => self.score_o += 1, _ => {} }
            self.state = GameState::Won(w);
            let _ = self.save_scores();
            return;
        }
        if self.board.iter().all(|c| *c != Cell::Empty) {
            self.draws += 1; self.state = GameState::Draw;
            let _ = self.save_scores();
            return;
        }
        self.current = ttt_flip(self.current);
        if self.mode == GameMode::VsAi && self.current == Cell::O {
            if let Some(i) = ttt_ai(&self.board) {
                self.board[i] = Cell::O;
                if let Some(w) = ttt_winner(&self.board) {
                    match w { Cell::X => self.score_x += 1, Cell::O => self.score_o += 1, _ => {} }
                    self.state = GameState::Won(w); return;
                }
                if self.board.iter().all(|c| *c != Cell::Empty) {
                    self.draws += 1; self.state = GameState::Draw; return;
                }
                self.current = Cell::X;
            }
        }
    }

    fn save_scores(&self) -> Result<(), String> {
        #[derive(Serialize, Deserialize)]
        struct TttScore { score_x: u32, score_o: u32, draws: u32 }
        let s = TttScore { score_x: self.score_x, score_o: self.score_o, draws: self.draws };
        GameHub::write_game_json("tictactoe_score.json", &s)
    }
}

// Вспомогательные типы для загрузки/сохранения
#[derive(Serialize, Deserialize)]
struct TicTacToeScore { score_x: u32, score_o: u32, draws: u32 }


fn ttt_flip(c: Cell) -> Cell { match c { Cell::X => Cell::O, Cell::O => Cell::X, _ => Cell::Empty } }

pub fn ttt_winner(board: &[Cell; 9]) -> Option<Cell> {
    const L: [[usize;3];8] = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];
    for [a,b,c] in L {
        if board[a]!=Cell::Empty && board[a]==board[b] && board[b]==board[c] { return Some(board[a]); }
    }
    None
}

fn ttt_ai(b: &[Cell; 9]) -> Option<usize> {
    if let Some(i) = ttt_win_move(b, Cell::O) { return Some(i); }
    if let Some(i) = ttt_win_move(b, Cell::X) { return Some(i); }
    if b[4]==Cell::Empty { return Some(4); }
    for &i in &[0,2,6,8] { if b[i]==Cell::Empty { return Some(i); } }
    b.iter().position(|c| *c==Cell::Empty)
}

fn ttt_win_move(b: &[Cell; 9], p: Cell) -> Option<usize> {
    const L: [[usize;3];8] = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];
    for [a,bb,c] in L {
        let cells = [b[a],b[bb],b[c]];
        if cells.iter().filter(|&&x| x==p).count()==2 {
            for (i, &pos) in [a,bb,c].iter().enumerate() { if cells[i]==Cell::Empty { return Some(pos); } }
        }
    }
    None
}

// ============================================================================
// Змейка
// ============================================================================

#[derive(Clone, Copy, PartialEq)]
pub enum Dir { Up, Down, Left, Right }

pub struct SnakeGame {
    pub body:       VecDeque<(i32, i32)>,
    pub dir:        Dir,
    pub next_dir:   Dir,
    pub food:       (i32, i32),
    pub cols:       i32,
    pub rows:       i32,
    pub dead:       bool,
    pub score:      u32,
    pub high_score: u32,
    pub step_ms:    u64,
    last_step:      Instant,
    rng:            u64,
}

impl SnakeGame {
    pub fn new() -> Self {
        let rng = now_nanos();
        let mut g = Self {
            body: VecDeque::new(), dir: Dir::Right, next_dir: Dir::Right,
            food: (10, 5), cols: 20, rows: 16,
            dead: false, score: 0, high_score: 0, step_ms: 150,
            last_step: Instant::now(), rng,
        };
        g.body.push_back((5, 8));
        g.body.push_back((4, 8));
        g.body.push_back((3, 8));
        g
    }

    pub fn reset(&mut self) {
        let hs = self.high_score.max(self.score);
        let rng = self.rng;
        *self = SnakeGame::new();
        self.high_score = hs;
        self.rng = rng;
    }

    pub fn set_dir(&mut self, d: Dir) {
        let bad = matches!((self.dir, d),
            (Dir::Up,Dir::Down)|(Dir::Down,Dir::Up)|(Dir::Left,Dir::Right)|(Dir::Right,Dir::Left));
        if !bad { self.next_dir = d; }
    }

    pub fn step(&mut self) -> bool {
        if self.dead || self.last_step.elapsed().as_millis() < self.step_ms as u128 { return false; }
        self.last_step = Instant::now();
        self.dir = self.next_dir;
        let head = *self.body.front().unwrap();
        let nh = match self.dir {
            Dir::Up    => (head.0, head.1-1),
            Dir::Down  => (head.0, head.1+1),
            Dir::Left  => (head.0-1, head.1),
            Dir::Right => (head.0+1, head.1),
        };
        if nh.0<0 || nh.0>=self.cols || nh.1<0 || nh.1>=self.rows || self.body.contains(&nh) {
            self.dead = true;
            let _ = self.save_score();
            return true;
        }
        self.body.push_front(nh);
        if nh == self.food {
            self.score += 1;
            self.high_score = self.high_score.max(self.score);
            self.step_ms = (150u64.saturating_sub(self.score as u64 * 3)).max(60);
            self.spawn_food();
        } else {
            self.body.pop_back();
        }
        true
    }

    fn rand(&mut self) -> u64 { self.rng = lcg(self.rng); self.rng }

    fn spawn_food(&mut self) {
        for _ in 0..500 {
            let x = (self.rand() % self.cols as u64) as i32;
            let y = (self.rand() % self.rows as u64) as i32;
            if !self.body.contains(&(x,y)) { self.food = (x,y); return; }
        }
    }

    fn save_score(&self) -> Result<(), String> {
        #[derive(Serialize, Deserialize)]
        struct SnakeScore { high_score: u32, last_score: u32 }
        let s = SnakeScore { high_score: self.high_score, last_score: self.score };
        GameHub::write_game_json("snake_score.json", &s)
    }
}

#[derive(Serialize, Deserialize)]
struct SnakeScore { high_score: u32, last_score: u32 }


// ============================================================================
// Тетрис
// ============================================================================

// [тип][поворот] — 4×4 битовая маска (MSB = верхний левый)
const TETROS: [[u16;4];7] = [
    [0x0F00,0x2222,0x00F0,0x4444], // I
    [0x0660,0x0660,0x0660,0x0660], // O
    [0x0E40,0x4C40,0x4E00,0x4640], // T
    [0x06C0,0x4620,0x06C0,0x4620], // S
    [0x0C60,0x2640,0x0C60,0x2640], // Z
    [0x0E80,0xC440,0x2E00,0x44C0], // J
    [0x0E20,0x4460,0x8E00,0xC440], // L
];

pub struct TetrisGame {
    pub board: [[u8;10];20],  // 0=пусто, 1-7=цвет
    pub ptype: usize,
    pub prot:  usize,
    pub px:    i32,
    pub py:    i32,
    pub next:  usize,
    pub score: u32,
    pub level: u32,
    pub lines: u32,
    pub over:  bool,
    last_fall: Instant,
    rng:       u64,
}

impl TetrisGame {
    pub fn new() -> Self {
        let mut g = Self {
            board: [[0;10];20], ptype:0, prot:0, px:3, py:0, next:0,
            score:0, level:1, lines:0, over:false,
            last_fall: Instant::now(), rng: now_nanos(),
        };
        g.ptype = (g.rand()%7) as usize;
        g.next  = (g.rand()%7) as usize;
        g
    }

    pub fn reset(&mut self) {
        let rng = self.rng;
        *self = TetrisGame::new();
        self.rng = rng;
    }

    fn rand(&mut self) -> u64 { self.rng = lcg(self.rng); self.rng }

    pub fn fall_ms(&self) -> u64 {
        (500u64.saturating_sub((self.level as u64 -1)*40)).max(60)
    }

    pub fn shape(ptype: usize, rot: usize) -> [(i32,i32);4] {
        let mask = TETROS[ptype][rot%4];
        let mut cells = [(0i32,0i32);4];
        let mut n = 0;
        for p in 0u16..16 {
            if mask & (0x8000>>p) != 0 { cells[n]=((p/4)as i32,(p%4)as i32); n+=1; }
        }
        cells
    }

    pub fn cells(&self) -> [(i32,i32);4] {
        Self::shape(self.ptype, self.prot).map(|(r,c)|(self.py+r, self.px+c))
    }

    fn valid(&self, cells: &[(i32,i32);4]) -> bool {
        for &(r,c) in cells {
            if c<0||c>=10||r>=20 { return false; }
            if r>=0 && self.board[r as usize][c as usize]!=0 { return false; }
        }
        true
    }

    pub fn left(&mut self) {
        let cc = Self::shape(self.ptype,self.prot).map(|(r,c)|(self.py+r,self.px-1+c));
        if self.valid(&cc) { self.px-=1; }
    }
    pub fn right(&mut self) {
        let cc = Self::shape(self.ptype,self.prot).map(|(r,c)|(self.py+r,self.px+1+c));
        if self.valid(&cc) { self.px+=1; }
    }
    pub fn rotate(&mut self) {
        let nr=(self.prot+1)%4;
        let cc=Self::shape(self.ptype,nr).map(|(r,c)|(self.py+r,self.px+c));
        if self.valid(&cc) { self.prot=nr; }
    }
    pub fn soft_drop(&mut self) {
        let cc=Self::shape(self.ptype,self.prot).map(|(r,c)|(self.py+1+r,self.px+c));
        if self.valid(&cc) { self.py+=1; }
    }
    pub fn hard_drop(&mut self) {
        while { let cc=Self::shape(self.ptype,self.prot).map(|(r,c)|(self.py+1+r,self.px+c)); self.valid(&cc) } {
            self.py+=1;
        }
        self.place();
    }

    pub fn ghost_y(&self) -> i32 {
        let mut y=self.py;
        while { let cc=Self::shape(self.ptype,self.prot).map(|(r,c)|(y+1+r,self.px+c)); self.valid(&cc) } { y+=1; }
        y
    }

    fn place(&mut self) {
        let color=(self.ptype+1) as u8;
        for (r,c) in self.cells() {
            if r>=0&&r<20&&c>=0&&c<10 { self.board[r as usize][c as usize]=color; }
        }
        self.clear_lines();
        self.spawn();
    }

    // Вызывается при окончании игры (game over) — сохраняем счёт
    fn on_game_over(&self) {
        let _ = self.save_score();
    }

    fn clear_lines(&mut self) {
        let mut cleared=0u32;
        let mut nb=[[0u8;10];20];
        let mut nr=19i32;
        for row in (0..20).rev() {
            if self.board[row].iter().any(|&c|c==0) {
                nb[nr as usize]=self.board[row]; nr-=1;
            } else { cleared+=1; }
        }
        if cleared>0 {
            self.board=nb; self.lines+=cleared;
            self.score += match cleared{1=>100,2=>300,3=>700,_=>1500}*self.level;
            self.level=(self.lines/10)+1;
        }
    }

    fn spawn(&mut self) {
        self.ptype=self.next; self.prot=0; self.px=3; self.py=0;
        self.next=(self.rand()%7) as usize;
        if !self.valid(&self.cells()) { 
            self.over=true; 
            let _ = self.save_score();
        }
        self.last_fall=Instant::now();
    }

    pub fn tick(&mut self) -> bool {
        if self.over || self.last_fall.elapsed().as_millis()<self.fall_ms() as u128 { return false; }
        self.last_fall=Instant::now();
        let cc=Self::shape(self.ptype,self.prot).map(|(r,c)|(self.py+1+r,self.px+c));
        if self.valid(&cc) { self.py+=1; false }
        else { self.place(); true }
    }

    
}

// ============================================================================
// Сапёр
// ============================================================================

pub const MS_ROWS: usize = 9;
pub const MS_COLS: usize = 9;
const MS_MINES: u32 = 10;

#[derive(Clone, Copy, PartialEq)]
pub enum MsState { Waiting, Playing, Won, Lost }

pub struct MinesweeperGame {
    pub mines:    [[bool; MS_COLS]; MS_ROWS],
    pub revealed: [[bool; MS_COLS]; MS_ROWS],
    pub flagged:  [[bool; MS_COLS]; MS_ROWS],
    pub state:    MsState,
    pub flags:    i32,
    pub hit:      Option<(usize, usize)>,  // клетка с миной, на которую наступили
    start_time:   Option<Instant>,
    pub elapsed:  u64,
    rng:          u64,
}

impl MinesweeperGame {
    pub fn new() -> Self {
        Self {
            mines:    [[false; MS_COLS]; MS_ROWS],
            revealed: [[false; MS_COLS]; MS_ROWS],
            flagged:  [[false; MS_COLS]; MS_ROWS],
            state:    MsState::Waiting,
            flags:    0,
            hit:      None,
            start_time: None,
            elapsed:  0,
            rng:      now_nanos(),
        }
    }

    pub fn reset(&mut self) {
        let rng = self.rng;
        *self = MinesweeperGame::new();
        self.rng = rng;
    }

    fn rand(&mut self) -> u64 { self.rng = lcg(self.rng); self.rng }

    fn place_mines(&mut self, avoid_col: usize, avoid_row: usize) {
        let mut placed = 0u32;
        for _ in 0..100_000 {
            if placed >= MS_MINES { break; }
            let c = (self.rand() % MS_COLS as u64) as usize;
            let r = (self.rand() % MS_ROWS as u64) as usize;
            // Не ставим мину рядом с первым кликом (3×3 зона)
            let dc = (c as i32 - avoid_col as i32).abs();
            let dr = (r as i32 - avoid_row as i32).abs();
            if dc <= 1 && dr <= 1 { continue; }
            if !self.mines[r][c] { self.mines[r][c] = true; placed += 1; }
        }
    }

    pub fn adjacent_mines(&self, col: usize, row: usize) -> u8 {
        let mut count = 0u8;
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                if dr == 0 && dc == 0 { continue; }
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                if nr >= 0 && nr < MS_ROWS as i32 && nc >= 0 && nc < MS_COLS as i32 {
                    if self.mines[nr as usize][nc as usize] { count += 1; }
                }
            }
        }
        count
    }

    pub fn reveal(&mut self, col: usize, row: usize) {
        if matches!(self.state, MsState::Won | MsState::Lost) { return; }
        if self.flagged[row][col] || self.revealed[row][col] { return; }

        // Первый клик — расставляем мины
        if self.state == MsState::Waiting {
            self.place_mines(col, row);
            self.state = MsState::Playing;
            self.start_time = Some(Instant::now());
        }

        if self.mines[row][col] {
            self.revealed[row][col] = true;
            self.hit = Some((col, row));
            self.state = MsState::Lost;
            // Открываем все мины
            for r in 0..MS_ROWS {
                for c in 0..MS_COLS {
                    if self.mines[r][c] { self.revealed[r][c] = true; }
                }
            }
            if let Some(t) = self.start_time { self.elapsed = t.elapsed().as_secs(); }
            let _ = self.save_result();
            return;
        }

        self.flood_fill(col, row);
        self.check_win();
    }

    fn flood_fill(&mut self, col: usize, row: usize) {
        if row >= MS_ROWS || col >= MS_COLS { return; }
        if self.revealed[row][col] || self.flagged[row][col] || self.mines[row][col] { return; }
        self.revealed[row][col] = true;
        if self.adjacent_mines(col, row) == 0 {
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 { continue; }
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    if nr >= 0 && nr < MS_ROWS as i32 && nc >= 0 && nc < MS_COLS as i32 {
                        self.flood_fill(nc as usize, nr as usize);
                    }
                }
            }
        }
    }

    fn check_win(&mut self) {
        for r in 0..MS_ROWS {
            for c in 0..MS_COLS {
                if !self.mines[r][c] && !self.revealed[r][c] { return; }
            }
        }
        self.state = MsState::Won;
        if let Some(t) = self.start_time { self.elapsed = t.elapsed().as_secs(); }
        let _ = self.save_result();
    }

    pub fn flag(&mut self, col: usize, row: usize) {
        if !matches!(self.state, MsState::Waiting | MsState::Playing) { return; }
        if self.revealed[row][col] { return; }
        if self.flagged[row][col] {
            self.flagged[row][col] = false;
            self.flags -= 1;
        } else {
            self.flagged[row][col] = true;
            self.flags += 1;
        }
    }

    pub fn mines_left(&self) -> i32 { MS_MINES as i32 - self.flags }

    pub fn elapsed_secs(&self) -> u64 {
        match self.state {
            MsState::Playing => self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0),
            _ => self.elapsed,
        }
    }

    fn save_result(&self) -> Result<(), String> {
        #[derive(Serialize)]
        struct MinesResult { state: u8, elapsed: u64 }
        let s = MinesResult { state: match self.state { MsState::Waiting=>0, MsState::Playing=>1, MsState::Won=>2, MsState::Lost=>3 }, elapsed: self.elapsed };
        GameHub::write_game_json("minesweeper_result.json", &s)
    }
}

// ============================================================================
// Вспомогательные функции
// ============================================================================

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn lcg(x: u64) -> u64 {
    x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

// ============================================================================
// Форматы сохранений
// ============================================================================

#[derive(Serialize, Deserialize)]
struct TicTacToeSave {
    board: [u8; 9],
    current: u8,
    state: u8, // 0=игра,1=победа_X,2=победа_O,3=ничья
    mode: u8,  // 0=VsAi,1=Два игрока
    score_x: u32,
    score_o: u32,
    draws: u32,
}

impl From<&TicTacToe> for TicTacToeSave {
    fn from(t: &TicTacToe) -> Self {
        let mut board = [0u8; 9];
        for (i, c) in t.board.iter().enumerate() {
            board[i] = match c { Cell::Empty => 0, Cell::X => 1, Cell::O => 2 };
        }
        let state = match t.state {
            GameState::Playing => 0,
            GameState::Won(Cell::X) => 1,
            GameState::Won(Cell::O) => 2,
            GameState::Won(_) => 0,
            GameState::Draw => 3,
        };
        let mode = match t.mode { GameMode::VsAi => 0, GameMode::TwoPlayers => 1 };
        let current = match t.current { Cell::Empty => 0, Cell::X => 1, Cell::O => 2 };
        TicTacToeSave { board, current, state, mode, score_x: t.score_x, score_o: t.score_o, draws: t.draws }
    }
}

impl TicTacToeSave {
    fn apply(&self, t: &mut TicTacToe) {
        for (i, v) in self.board.iter().enumerate() {
            t.board[i] = match v { 0 => Cell::Empty, 1 => Cell::X, 2 => Cell::O, _ => Cell::Empty };
        }
        t.current = match self.current { 1 => Cell::X, 2 => Cell::O, _ => Cell::X };
        t.state = match self.state {
            1 => GameState::Won(Cell::X),
            2 => GameState::Won(Cell::O),
            3 => GameState::Draw,
            _ => GameState::Playing,
        };
        t.mode = if self.mode == 1 { GameMode::TwoPlayers } else { GameMode::VsAi };
        t.score_x = self.score_x;
        t.score_o = self.score_o;
        t.draws = self.draws;
    }
}

#[derive(Serialize, Deserialize)]
struct SnakeSave {
    body: Vec<(i32,i32)>,
    dir: u8,
    next_dir: u8,
    food: (i32,i32),
    cols: i32,
    rows: i32,
    dead: bool,
    score: u32,
    high_score: u32,
    step_ms: u64,
    rng: u64,
}

impl From<&SnakeGame> for SnakeSave {
    fn from(s: &SnakeGame) -> Self {
        SnakeSave {
            body: s.body.iter().cloned().collect(),
            dir: match s.dir { Dir::Up=>0, Dir::Down=>1, Dir::Left=>2, Dir::Right=>3 },
            next_dir: match s.next_dir { Dir::Up=>0, Dir::Down=>1, Dir::Left=>2, Dir::Right=>3 },
            food: s.food,
            cols: s.cols,
            rows: s.rows,
            dead: s.dead,
            score: s.score,
            high_score: s.high_score,
            step_ms: s.step_ms,
            rng: s.rng,
        }
    }
}

impl SnakeSave {
    fn apply(&self, s: &mut SnakeGame) {
        s.body = self.body.iter().cloned().collect();
        s.dir = match self.dir { 0=>Dir::Up, 1=>Dir::Down, 2=>Dir::Left, _=>Dir::Right };
        s.next_dir = match self.next_dir { 0=>Dir::Up, 1=>Dir::Down, 2=>Dir::Left, _=>Dir::Right };
        s.food = self.food;
        s.cols = self.cols;
        s.rows = self.rows;
        s.dead = self.dead;
        s.score = self.score;
        s.high_score = self.high_score;
        s.step_ms = self.step_ms;
        s.rng = self.rng;
        // обновляем `last_step` на текущее время
        s.last_step = Instant::now();
    }
}

#[derive(Serialize, Deserialize)]
struct TetrisSave {
    board: [[u8;10];20],
    ptype: usize,
    prot: usize,
    px: i32,
    py: i32,
    next: usize,
    score: u32,
    level: u32,
    lines: u32,
    over: bool,
    rng: u64,
}

impl From<&TetrisGame> for TetrisSave {
    fn from(t: &TetrisGame) -> Self {
        TetrisSave { board: t.board, ptype: t.ptype, prot: t.prot, px: t.px, py: t.py, next: t.next, score: t.score, level: t.level, lines: t.lines, over: t.over, rng: t.rng }
    }
}

impl TetrisSave {
    fn apply(&self, t: &mut TetrisGame) {
        t.board = self.board;
        t.ptype = self.ptype;
        t.prot = self.prot;
        t.px = self.px;
        t.py = self.py;
        t.next = self.next;
        t.score = self.score;
        t.level = self.level;
        t.lines = self.lines;
        t.over = self.over;
        t.rng = self.rng;
        t.last_fall = Instant::now();
    }
}

#[derive(Serialize, Deserialize)]
struct TetrisScore { score: u32, level: u32, lines: u32 }

impl TetrisGame {
    fn save_score(&self) -> Result<(), String> {
        let s = TetrisScore { score: self.score, level: self.level, lines: self.lines };
        GameHub::write_game_json("tetris_score.json", &s)
    }
}

#[derive(Serialize, Deserialize)]
struct MinesSave {
    mines: [[bool; MS_COLS]; MS_ROWS],
    revealed: [[bool; MS_COLS]; MS_ROWS],
    flagged: [[bool; MS_COLS]; MS_ROWS],
    state: u8, // 0=ожидание,1=игра,2=победа,3=поражение
    flags: i32,
    hit: Option<(usize,usize)>,
    elapsed: u64,
    rng: u64,
}

impl From<&MinesweeperGame> for MinesSave {
    fn from(m: &MinesweeperGame) -> Self {
        MinesSave {
            mines: m.mines,
            revealed: m.revealed,
            flagged: m.flagged,
            state: match m.state { MsState::Waiting=>0, MsState::Playing=>1, MsState::Won=>2, MsState::Lost=>3 },
            flags: m.flags,
            hit: m.hit,
            elapsed: m.elapsed,
            rng: m.rng,
        }
    }
}

impl MinesSave {
    fn apply(&self, m: &mut MinesweeperGame) {
        m.mines = self.mines;
        m.revealed = self.revealed;
        m.flagged = self.flagged;
        m.state = match self.state { 0=>MsState::Waiting, 1=>MsState::Playing, 2=>MsState::Won, _=>MsState::Lost };
        m.flags = self.flags;
        m.hit = self.hit;
        m.elapsed = self.elapsed;
        m.rng = self.rng;
        m.start_time = None;
    }
}

#[derive(Serialize, Deserialize)]
struct MinesResult { state: u8, elapsed: u64 }
