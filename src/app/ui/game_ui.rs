//! Рендеринг мини-игр

use crate::app::game::{
    ActiveGame, Cell, Dir, GameMode, GameState, MsState, TetrisGame, MS_COLS, MS_ROWS,
};
use eframe::egui;
use std::time::Duration;

const TC: f32 = 22.0; // Тетрис: размер клетки
const SC: f32 = 17.0; // Змейка: размер клетки
const MC: f32 = 28.0; // Сапёр: размер клетки

pub fn render(ctx: &egui::Context, app: &mut crate::app::AssistantApp, accent: egui::Color32) {
    // Запрашиваем перерисовку для анимированных игр
    if app.show_game {
        let repaint_ms = match app.games.active {
            ActiveGame::Snake if !app.games.snake.dead => Some(app.games.snake.step_ms),
            ActiveGame::Tetris if !app.games.tetris.over => Some(app.games.tetris.fall_ms()),
            ActiveGame::Minesweeper if app.games.minesweeper.state == MsState::Playing => Some(500),
            _ => None,
        };
        if let Some(ms) = repaint_ms {
            ctx.request_repaint_after(Duration::from_millis(ms));
        }
    }

    egui::Window::new("Игры")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -80.0))
        .show(ctx, |ui| {
            // Вкладки выбора игры
            ui.horizontal(|ui| {
                for (label, game) in [
                    ("X/O",    ActiveGame::TicTacToe),
                    ("Змейка", ActiveGame::Snake),
                    ("Тетрис", ActiveGame::Tetris),
                    ("Сапёр",  ActiveGame::Minesweeper),
                ] {
                    let active = app.games.active == game;
                    let text = egui::RichText::new(label).size(14.0);
                    let btn = egui::Button::new(if active { text.strong().color(accent) } else { text })
                        .min_size(egui::vec2(50.0, 26.0));
                    if ui.add(btn).clicked() {
                        app.games.active = game;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("X").on_hover_text("Закрыть").clicked() {
                        app.show_game = false;
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            match app.games.active {
                ActiveGame::TicTacToe   => render_ttt(ui, app, accent),
                ActiveGame::Snake       => render_snake(ui, ctx, app, accent),
                ActiveGame::Tetris      => render_tetris(ui, ctx, app, accent),
                ActiveGame::Minesweeper => render_minesweeper(ui, app, accent),
            }
        });
}

// ============================================================================
// Крестики-нолики
// ============================================================================

fn render_ttt(ui: &mut egui::Ui, app: &mut crate::app::AssistantApp, accent: egui::Color32) {
    let g = &mut app.games.tictactoe;

    // Счёт
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("X: {}", g.score_x)).strong().color(accent));
        ui.label(egui::RichText::new(format!("Ничьи: {}", g.draws)).color(egui::Color32::GRAY));
        ui.label(egui::RichText::new(format!("O: {}", g.score_o)).strong().color(egui::Color32::LIGHT_RED));
    });

    let (status, scol) = match &g.state {
        GameState::Playing => (
            match (g.current, g.mode) {
                (Cell::X, _) => "Ход: X (вы)".to_string(),
                (Cell::O, GameMode::VsAi) => "Ход: O (AI)".to_string(),
                (Cell::O, GameMode::TwoPlayers) => "Ход: O".to_string(),
                _ => String::new(),
            },
            egui::Color32::WHITE,
        ),
        GameState::Won(Cell::X) => ("Победа X!".to_string(), accent),
        GameState::Won(Cell::O) => ("Победа O!".to_string(), egui::Color32::LIGHT_RED),
        GameState::Draw => ("Ничья!".to_string(), egui::Color32::YELLOW),
        GameState::Won(_) => (String::new(), egui::Color32::WHITE),
    };
    ui.label(egui::RichText::new(&status).strong().color(scol));
    ui.add_space(6.0);

    let csz = egui::vec2(72.0, 72.0);
    let playing = matches!(g.state, GameState::Playing);
    let mut click: Option<usize> = None;

    egui::Grid::new("ttt").spacing([4.0, 4.0]).show(ui, |ui| {
        for row in 0..3 {
            for col in 0..3 {
                let idx = row*3+col;
                let (txt, col_) = match g.board[idx] {
                    Cell::X => ("X", accent),
                    Cell::O => ("O", egui::Color32::LIGHT_RED),
                    Cell::Empty => ("", egui::Color32::GRAY),
                };
                let sense = if g.board[idx]==Cell::Empty && playing { egui::Sense::click() } else { egui::Sense::hover() };
                let btn = egui::Button::new(egui::RichText::new(txt).size(32.0).strong().color(col_))
                    .min_size(csz).sense(sense);
                if ui.add(btn).clicked() { click = Some(idx); }
            }
            ui.end_row();
        }
    });

    if let Some(idx) = click { g.make_move(idx); }
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("Заново").fill(accent).min_size(egui::vec2(80.0,26.0))).clicked() {
            g.reset();
        }
        let lbl = match g.mode { GameMode::VsAi => "2 игрока", GameMode::TwoPlayers => "vs AI" };
        if ui.button(egui::RichText::new(lbl).size(12.0)).on_hover_text("Переключить режим").clicked() {
            g.mode = match g.mode { GameMode::VsAi => GameMode::TwoPlayers, GameMode::TwoPlayers => GameMode::VsAi };
            g.score_x=0; g.score_o=0; g.draws=0; g.reset();
        }
    });
}

// ============================================================================
// Змейка
// ============================================================================

fn render_snake(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut crate::app::AssistantApp, accent: egui::Color32) {
    // Клавиши (только если нет фокуса в текстовых полях)
    if !ctx.memory(|m| m.focus().is_some()) {
        ctx.input(|i| {
            let g = &mut app.games.snake;
            if i.key_pressed(egui::Key::ArrowUp)    || i.key_pressed(egui::Key::W) { g.set_dir(Dir::Up); }
            if i.key_pressed(egui::Key::ArrowDown)  || i.key_pressed(egui::Key::S) { g.set_dir(Dir::Down); }
            if i.key_pressed(egui::Key::ArrowLeft)  || i.key_pressed(egui::Key::A) { g.set_dir(Dir::Left); }
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::D) { g.set_dir(Dir::Right); }
        });
    }

    app.games.snake.step();

    let g = &app.games.snake;

    // Счёт
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("Счёт: {}", g.score)).strong().color(accent));
        ui.label(egui::RichText::new(format!("Рекорд: {}", g.high_score)).color(egui::Color32::GRAY));
    });
    if g.dead {
        ui.label(egui::RichText::new("Игра окончена!").strong().color(egui::Color32::RED));
    }
    ui.add_space(4.0);

    // Поле
    let board_size = egui::vec2(g.cols as f32 * SC, g.rows as f32 * SC);
    let (rect, _) = ui.allocate_exact_size(board_size, egui::Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 2.0, egui::Color32::from_rgb(10,20,10));

    // Сетка
    for col in 0..g.cols {
        for row in 0..g.rows {
            let cr = cell_rect_f(rect.min, col as f32, row as f32, SC);
            p.rect_stroke(cr, 0.0, egui::Stroke::new(0.3, egui::Color32::from_rgb(20,40,20)));
        }
    }

    // Еда
    let fr = cell_rect_f(rect.min, g.food.0 as f32, g.food.1 as f32, SC);
    p.rect_filled(fr, SC*0.3, egui::Color32::from_rgb(220,60,60));

    // Тело
    for (i, &(cx,cy)) in g.body.iter().enumerate() {
        let cr = cell_rect_f(rect.min, cx as f32, cy as f32, SC);
        let color = if i==0 { egui::Color32::from_rgb(100,220,80) }
                    else { egui::Color32::from_rgb(60,160,50) };
        p.rect_filled(cr, 2.0, color);
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("Заново").fill(accent).min_size(egui::vec2(80.0,26.0))).clicked() {
            app.games.snake.reset();
        }
        ui.label(egui::RichText::new("WASD / стрелки").color(egui::Color32::GRAY).small());
    });
}

// ============================================================================
// Тетрис
// ============================================================================

fn render_tetris(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut crate::app::AssistantApp, accent: egui::Color32) {
    if !ctx.memory(|m| m.focus().is_some()) {
        ctx.input(|i| {
            let g = &mut app.games.tetris;
            if !g.over {
                if i.key_pressed(egui::Key::ArrowLeft)  { g.left(); }
                if i.key_pressed(egui::Key::ArrowRight) { g.right(); }
                if i.key_pressed(egui::Key::ArrowUp)    { g.rotate(); }
                if i.key_pressed(egui::Key::ArrowDown)  { g.soft_drop(); }
                if i.key_pressed(egui::Key::Space)      { g.hard_drop(); }
            }
        });
    }

    app.games.tetris.tick();
    let mut do_reset = false;

    {
    let g = &app.games.tetris;
    ui.horizontal(|ui| {
        // Игровое поле
        ui.vertical(|ui| {
            let board_size = egui::vec2(10.0*TC, 20.0*TC);
            let (rect, _) = ui.allocate_exact_size(board_size, egui::Sense::hover());
            let p = ui.painter_at(rect);
            p.rect_filled(rect, 2.0, egui::Color32::from_rgb(8,8,16));

            // Сетка
            for r in 0..20i32 {
                for c in 0..10i32 {
                    let cr = cell_rect_f(rect.min, c as f32, r as f32, TC);
                    p.rect_stroke(cr, 0.0, egui::Stroke::new(0.4, egui::Color32::from_rgb(25,25,40)));
                }
            }

            // Заполненные клетки
            for r in 0..20usize {
                for c in 0..10usize {
                    if g.board[r][c] > 0 {
                        let cr = cell_rect_f(rect.min, c as f32, r as f32, TC);
                        p.rect_filled(cr, 2.0, tcolor(g.board[r][c] as usize - 1));
                    }
                }
            }

            // Призрак
            let gy = g.ghost_y();
            if gy != g.py {
                for (r,c) in TetrisGame::shape(g.ptype, g.prot) {
                    let (br,bc)=(gy+r, g.px+c);
                    if br>=0&&br<20&&bc>=0&&bc<10 {
                        let cr=cell_rect_f(rect.min, bc as f32, br as f32, TC);
                        let mut col=tcolor(g.ptype); col=egui::Color32::from_rgba_unmultiplied(col.r(),col.g(),col.b(),50);
                        p.rect_filled(cr, 2.0, col);
                    }
                }
            }

            // Текущая фигура
            for (r,c) in g.cells() {
                if r>=0&&r<20&&c>=0&&c<10 {
                    let cr=cell_rect_f(rect.min, c as f32, r as f32, TC);
                    p.rect_filled(cr, 2.0, tcolor(g.ptype));
                }
            }
        });

        ui.add_space(8.0);

        // Панель: следующая + счёт
        ui.vertical(|ui| {
            ui.set_min_width(90.0);
            ui.label(egui::RichText::new("Следующая:").small().color(egui::Color32::GRAY));
            let np_size = egui::vec2(4.0*TC, 4.0*TC);
            let (rect2, _) = ui.allocate_exact_size(np_size, egui::Sense::hover());
            let p2 = ui.painter_at(rect2);
            p2.rect_filled(rect2, 2.0, egui::Color32::from_rgb(8,8,16));
            for (r,c) in TetrisGame::shape(g.next, 0) {
                let cr=cell_rect_f(rect2.min, c as f32, r as f32, TC);
                p2.rect_filled(cr, 2.0, tcolor(g.next));
            }

            ui.add_space(10.0);
            ui.label(egui::RichText::new(format!("Очки\n{}", g.score)).strong().color(accent));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(format!("Уровень\n{}", g.level)).color(egui::Color32::WHITE));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(format!("Линии\n{}", g.lines)).color(egui::Color32::GRAY));

            if g.over {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("GAME\nOVER").strong().color(egui::Color32::RED));
            }

            ui.add_space(10.0);
            if ui.add(egui::Button::new("Заново").fill(accent).min_size(egui::vec2(80.0,26.0))).clicked() {
                do_reset = true;
            }
            ui.add_space(4.0);
            ui.label(egui::RichText::new("<> движение\n^ поворот\nv мягко\nSpc бросить").size(10.0).color(egui::Color32::GRAY));
        });
    });
    } // конец блока borrow g

    if do_reset { app.games.tetris.reset(); }
}

// ============================================================================
// Сапёр
// ============================================================================

fn render_minesweeper(ui: &mut egui::Ui, app: &mut crate::app::AssistantApp, accent: egui::Color32) {
    let g = &app.games.minesweeper;

    // Шапка: мины и таймер
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("Мины: {:2}", g.mines_left())).strong().color(accent));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(format!("{:3}s", g.elapsed_secs())).color(egui::Color32::GRAY));
        });
    });

    let status_text = match g.state {
        MsState::Won     => Some(("Поле разминировано!", egui::Color32::LIGHT_GREEN)),
        MsState::Lost    => Some(("Подрыв! Игра окончена.", egui::Color32::RED)),
        MsState::Waiting => Some(("Кликните чтобы начать", egui::Color32::GRAY)),
        MsState::Playing => None,
    };
    if let Some((txt, col)) = status_text {
        ui.label(egui::RichText::new(txt).color(col));
    } else {
        ui.label("");  // держим высоту постоянной
    }
    ui.add_space(4.0);

    // Поле
    let size = egui::vec2(MS_COLS as f32 * MC, MS_ROWS as f32 * MC);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 4.0, egui::Color32::from_rgb(28, 28, 38));

    // Определяем клетку под курсором один раз
    let hover_cell = resp.hover_pos().and_then(|pos| {
        let c = ((pos.x - rect.min.x) / MC) as usize;
        let r = ((pos.y - rect.min.y) / MC) as usize;
        if c < MS_COLS && r < MS_ROWS { Some((c, r)) } else { None }
    });

    // Обрабатываем клики до рендера клеток
    if let Some((hc, hr)) = hover_cell {
        if resp.clicked() {
            app.games.minesweeper.reveal(hc, hr);
        }
        if resp.secondary_clicked() {
            app.games.minesweeper.flag(hc, hr);
        }
    }

    // Рисуем клетки после возможной мутации
    let g = &app.games.minesweeper;
    for row in 0..MS_ROWS {
        for col in 0..MS_COLS {
            let cr = egui::Rect::from_min_size(
                rect.min + egui::vec2(col as f32 * MC, row as f32 * MC),
                egui::vec2(MC - 1.0, MC - 1.0),
            );
            let is_hover = matches!(g.state, MsState::Waiting | MsState::Playing)
                           && hover_cell == Some((col, row));

            if g.revealed[row][col] {
                // Открытая клетка
                let bg = if g.hit == Some((col, row)) {
                    egui::Color32::from_rgb(200, 30, 30)  // мина, на которую наступили
                } else if g.mines[row][col] {
                    egui::Color32::from_rgb(110, 40, 40)  // другие мины (поражение)
                } else {
                    egui::Color32::from_rgb(45, 45, 60)
                };
                p.rect_filled(cr, 2.0, bg);

                if g.mines[row][col] {
                    p.text(cr.center(), egui::Align2::CENTER_CENTER, "*",
                           egui::FontId::proportional(18.0), egui::Color32::WHITE);
                } else {
                    let n = g.adjacent_mines(col, row);
                    if n > 0 {
                        p.text(cr.center(), egui::Align2::CENTER_CENTER, &n.to_string(),
                               egui::FontId::proportional(15.0), ms_num_color(n));
                    }
                }
            } else if g.flagged[row][col] {
                p.rect_filled(cr, 2.0, egui::Color32::from_rgb(50, 30, 70));
                p.text(cr.center(), egui::Align2::CENTER_CENTER, "F",
                       egui::FontId::proportional(14.0), egui::Color32::from_rgb(220, 80, 80));
            } else {
                // Скрытая клетка
                let bg = if is_hover {
                    egui::Color32::from_rgb(75, 85, 105)
                } else {
                    egui::Color32::from_rgb(55, 65, 85)
                };
                p.rect_filled(cr, 2.0, bg);
            }
        }
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("Новая игра").fill(accent).min_size(egui::vec2(90.0, 26.0))).clicked() {
            app.games.minesweeper.reset();
        }
        ui.label(egui::RichText::new("ЛКМ — открыть  ПКМ — флаг").color(egui::Color32::GRAY).small());
    });
}

fn ms_num_color(n: u8) -> egui::Color32 {
    match n {
        1 => egui::Color32::from_rgb(80, 130, 255),
        2 => egui::Color32::from_rgb(60, 190, 80),
        3 => egui::Color32::from_rgb(230, 60, 60),
        4 => egui::Color32::from_rgb(110, 60, 210),
        5 => egui::Color32::from_rgb(210, 80, 40),
        6 => egui::Color32::from_rgb(50, 210, 200),
        7 => egui::Color32::from_rgb(200, 60, 200),
        _ => egui::Color32::from_rgb(160, 160, 160),
    }
}

// ============================================================================
// Вспомогательные
// ============================================================================

fn cell_rect_f(origin: egui::Pos2, col: f32, row: f32, sz: f32) -> egui::Rect {
    egui::Rect::from_min_size(origin + egui::vec2(col*sz, row*sz), egui::vec2(sz-1.0, sz-1.0))
}

fn tcolor(piece: usize) -> egui::Color32 {
    match piece % 7 {
        0 => egui::Color32::from_rgb(0,220,220),
        1 => egui::Color32::from_rgb(220,220,0),
        2 => egui::Color32::from_rgb(160,0,220),
        3 => egui::Color32::from_rgb(0,200,0),
        4 => egui::Color32::from_rgb(220,0,0),
        5 => egui::Color32::from_rgb(0,80,220),
        _ => egui::Color32::from_rgb(220,140,0),
    }
}
