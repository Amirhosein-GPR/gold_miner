use ggez::{graphics, Context, GameResult};
use ggez::event::EventHandler;
use rand::Rng;
use std::fs::File;
use std::io::{Read};

#[derive(Clone)]
struct Vector2D {
    x: f32,
    y: f32
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y
        }
    }
}

impl Into<ggez::mint::Point2<f32>> for Vector2D {
    fn into(self) -> ggez::mint::Point2<f32> {
        ggez::mint::Point2::<f32> {
            x: self.x,
            y: self.y
        }
    }
}

struct Cell {
    mesh: graphics::Mesh,
}

impl Cell {
    fn new(mesh: graphics::Mesh) -> Self {
        Self {
            mesh,
        }
    }
}

struct NumberText {
    text: graphics::Text
}

impl NumberText {
    pub fn new(text: graphics::Text) -> Self {
        Self {
            text
        }
    }
}

struct HeadNode {
    mesh: graphics::Mesh,
    start_row: u8,
    location: [u8; 2],
    path: Vec<[u8; 2]>,
    locked_location: [u8; 2],
    game_not_finished: bool
}

impl HeadNode {
    pub fn new(mesh: graphics::Mesh) -> Self {
        let mut path: Vec<[u8; 2]> = Vec::new();
        path.push([0, 0]);
        Self {
            mesh,
            start_row: 0,
            location: [0, 0],
            path,
            locked_location: [10, 10],
            game_not_finished: true
        }
    }

    pub fn new_move(&mut self, table: [u8; 2], numbers: &mut Vec<Vec<u8>>, menu_sum_numbers: &mut Vec<u8>, current_in_rows_cells_location: &mut Vec<Vec<[u8; 2]>>) -> [u8; 2] {
        println!("Head cell path : {:?}", self.path);
        if self.location[0] as i8 - 1 > -1 && self.location[1] + 1 < table[1] && self.locked_location != [self.location[0] - 1, self.location[1] + 1] && self.locked_location != [self.location[0], self.location[1] + 1] && self.locked_location != [self.location[0] + 1, self.location[1] + 1] {
            self.location[0] -= 1;
            self.location[1] += 1;
            self.path.push([self.location[0], self.location[1]]);
            menu_sum_numbers.push(*numbers.
                get_mut(self.location[0] as usize).
                unwrap().
                get_mut(self.location[1] as usize).
                unwrap()
            );
            current_in_rows_cells_location[self.start_row as usize].push(self.location);
        } else if self.location[1] + 1 < table[1] && self.locked_location != [self.location[0], self.location[1] + 1] && self.locked_location != [self.location[0] + 1, self.location[1] + 1] {
            self.location[1] += 1;
            self.path.push([self.location[0], self.location[1]]);
            menu_sum_numbers.push(*numbers.
                get_mut(self.location[0] as usize).
                unwrap().
                get_mut(self.location[1] as usize).
                unwrap()
            );
            current_in_rows_cells_location[self.start_row as usize].push(self.location);
        } else if self.location[0] + 1 < table[0] && self.location[1] + 1 < table[1] && self.locked_location != [self.location[0] + 1, self.location[1] + 1] {
            self.location[0] += 1;
            self.location[1] += 1;
            self.path.push([self.location[0], self.location[1]]);
            menu_sum_numbers.push(*numbers.
                get_mut(self.location[0] as usize).
                unwrap().
                get_mut(self.location[1] as usize).
                unwrap()
            );
            current_in_rows_cells_location[self.start_row as usize].push(self.location);
        } else {
            self.locked_location = [self.location[0], self.location[1]];
            self.path.pop().expect("Can't pop from path");
            menu_sum_numbers.pop().expect("Can't pop from menu_sum_numbers");
            current_in_rows_cells_location[self.start_row as usize].pop().expect("Can't pop from maximum_in_rows_cells_location");

            if self.path.len() != 0 {
                self.location = *self.path.last().unwrap();
            } else {
                if self.start_row == table[0] - 1 {
                    self.game_not_finished = false;
                } else {
                    self.start_row += 1;
                    self.location = [self.start_row, 0];
                    self.path.push([self.start_row, 0]);
                    menu_sum_numbers.push(*numbers.
                        get_mut(self.location[0] as usize).
                        unwrap().
                        get_mut(self.location[1] as usize).
                        unwrap()
                    );
                }
            }
        }
        self.location
    }
}

pub struct GoldMiner {
    table: [u8; 2],
    menu_background: graphics::Mesh,
    menu_texts: Vec<NumberText>,
    menu_sum_texts: Vec<NumberText>,
    menu_max_text: NumberText,
    menu_sum_numbers: Vec<u8>,
    menu_lines: Vec<graphics::Mesh>,
    cells: Vec<Vec<Cell>>,
    cell_width: f32,
    cell_height: f32,
    borders: Vec<Vec<Cell>>,
    numbers: Vec<Vec<u8>>,
    texts: Vec<Vec<NumberText>>,
    head_node: HeadNode,
    maximum_in_rows: Vec<u16>,
    current_in_rows_cells_location: Vec<Vec<[u8; 2]>>,
    maximum_in_rows_cells_location: Vec<Vec<[u8; 2]>>,
    current_milisec: u64,
    next_milisec: u64,
    refresh_rate_in_miliseconds: u64,
    game_not_finished: bool
}

impl GoldMiner {
    pub fn new(context: &mut Context) -> Self {
        let game_not_finished: bool = true;
        let mut settings = String::new();
        File::open("./settings.conf").expect("Error opening the settings.conf file").read_to_string(&mut settings).expect("Error reading to string");

        let settings_rows = settings.split('\n').collect::<Vec<&str>>();

        let table_size = settings_rows.get(0)
            .unwrap()
            .split(':').collect::<Vec<&str>>().get(1).unwrap().split('x').collect::<Vec<&str>>();
        
        let mut table: [u8; 2] = [0; 2];
        table[0] = table_size.get(0).unwrap().parse::<u8>().unwrap();
        table[1] = table_size.get(1).unwrap().parse::<u8>().unwrap();

        let max_number = settings_rows
        .get(1)
        .unwrap()
        .split(':')
        .collect::<Vec<&str>>();
        let max_number = max_number.get(1).unwrap();
        let max_number = max_number.parse::<u8>().unwrap();

        let refresh_rate_in_miliseconds = settings_rows
            .get(2)
            .unwrap()
            .split(':')
            .collect::<Vec<&str>>();
        let refresh_rate_in_miliseconds = refresh_rate_in_miliseconds.get(1).unwrap();
        let refresh_rate_in_miliseconds: u64 = refresh_rate_in_miliseconds.parse::<u64>().unwrap();

        let wait_then_solve_in_miliseconds = settings_rows
            .get(3)
            .unwrap()
            .split(':')
            .collect::<Vec<&str>>();
        let wait_then_solve_in_miliseconds = wait_then_solve_in_miliseconds.get(1).unwrap();
        let wait_then_solve_in_miliseconds: u64 = wait_then_solve_in_miliseconds.parse::<u64>().unwrap();

        let cell_width = 720.0 / table[1] as f32;
        let cell_height = 720.0 / table[0] as f32;

        let rect = graphics::Rect::new(0.0, 0.0, 560.0, 720.0);
        let menu_background = graphics::Mesh::new_rectangle(context, graphics::DrawMode::fill(), rect, graphics::Color::new(0.2, 0.2, 0.2, 1.0)).unwrap();
        let mut menu_texts: Vec<NumberText> = Vec::new();
        let mut menu_sum_texts: Vec<NumberText> = Vec::new();
        let menu_max_text: NumberText = NumberText::new(
            graphics::Text::new(
                graphics::TextFragment::new(
                    "Final Maximum : "
                ).
                font(graphics::Font::default()).
                color(graphics::Color::new(1.0, 1.0, 0.0, 1.0)).
                scale(graphics::PxScale::from(15.0))
            )
        );
        let mut menu_sum_numbers: Vec<u8> = Vec::new();
        let mut menu_lines: Vec<graphics::Mesh> = Vec::new();
        let mut cells: Vec<Vec<Cell>> = Vec::new();
        let mut cell_rect = graphics::Rect::new(0.0, 0.0, cell_width, cell_height);
        let mut borders: Vec<Vec<Cell>> = Vec::new();
        let mut numbers: Vec<Vec<u8>> = Vec::new();
        let mut texts: Vec<Vec<NumberText>> = Vec::new();

        for i in 0..table[0] {
            let mut color = graphics::Color::new(1.0, 0.0, 0.0, 1.0);
            match i {
                0 => {
                    color = graphics::Color::new(1.0, 0.0, 0.0, 1.0);
                },
                1 => {
                    color = graphics::Color::new(1.0, 0.5, 0.0, 1.0);
                },
                2 => {
                    color = graphics::Color::new(0.7, 0.7, 0.0, 1.0);
                },
                3 => {
                    color = graphics::Color::new(0.0, 1.0, 0.0, 1.0);
                },
                4 => {
                    color = graphics::Color::new(0.0, 0.0, 1.0, 1.0);
                },
                5 => {
                    color = graphics::Color::new(0.5, 0.0, 1.0, 1.0);
                },
                6 => {
                    color = graphics::Color::new(1.0, 0.0, 1.0, 1.0);
                },
                7 => {
                    color = graphics::Color::new(1.0, 1.0, 1.0, 1.0);
                },
                8 => {
                    color = graphics::Color::new(0.5, 0.5, 0.5, 1.0);
                },
                9 => {
                    color = graphics::Color::new(0.0, 0.0, 0.0, 1.0);
                },
                _ => {}
            }
            menu_texts.push(NumberText::new(
                graphics::Text::new(
                    graphics::TextFragment::new(
                        "Max : 0 ->"
                    ).
                    font(graphics::Font::default()).
                    color(color).
                    scale(graphics::PxScale::from(15.0)))
            ));
        }
        
        for i in 0..table[0] + 1 {
            menu_lines.push(graphics::Mesh::new_line(context, &[Vector2D::new(0.0, i as f32 * 720.0 / table[0] as f32), Vector2D::new(560.0, i as f32 * 720.0 / table[0] as f32)], 2.0, graphics::Color::WHITE).unwrap());
        }
        menu_lines.push(graphics::Mesh::new_line(context, &[Vector2D::new(0.0, 0.0), Vector2D::new(0.0, 720.0)], 2.0, graphics::Color::WHITE).unwrap());

        for i in 0..table[0] as usize {
            cells.push(Vec::new());
            cell_rect.y = i as f32 * cell_height;
            for j in 0..table[1] as usize {
                cell_rect.x = 560.0 + j as f32 * cell_width;
                cells.get_mut(i).unwrap().push(Cell::new(
                    graphics::Mesh::new_rectangle(context,
                        graphics::DrawMode::fill(),
                        cell_rect,
                        graphics::Color::new(0.2, 0.2, 0.2, 1.0)
                    ).unwrap()
                ));
            }
        }

        for i in 0..table[0] as usize {
            borders.push(Vec::new());
            cell_rect.y = i as f32 * cell_height;
            for j in 0..table[1] as usize {
                cell_rect.x = 560.0 + j as f32 * cell_width;
                borders.get_mut(i).unwrap().push(Cell::new(
                    graphics::Mesh::new_rectangle(context,
                        graphics::DrawMode::stroke(2.0),
                        cell_rect,
                        graphics::Color::WHITE
                    ).unwrap()
                ));
            }
        }

        for i in 0..table[0] as usize {
            texts.push(Vec::new());
            numbers.push(Vec::new());
            for _j in 0..table[1] as usize {
                let random_number: u8 = rand::thread_rng().gen_range(0..max_number);
                numbers.get_mut(i).unwrap().push(random_number);
                texts.get_mut(i).unwrap().push(NumberText::new(
                    graphics::Text::new(
                        graphics::TextFragment::new(
                        random_number.
                            to_string()
                        ).
                        font(graphics::Font::default()).
                        color(graphics::Color::new(1.0, 1.0, 0.0, 1.0)).
                        scale(graphics::PxScale::from(40.0)))
                ));
            }
        }

        let head_node = HeadNode::new(
            graphics::Mesh::new_circle(
                context,
                graphics::DrawMode::fill(),
                Vector2D::new(576.0 + cell_width / 2.5, cell_height / 1.2),
                3.0,
                0.5,
                graphics::Color::new(1.0, 0.0, 0.0, 1.0)
            ).unwrap()
        );

        let mut maximum_in_rows: Vec<u16> = Vec::new();
        let mut current_in_rows_cells_location: Vec<Vec<[u8; 2]>> = Vec::new();
        let mut maximum_in_rows_cells_location: Vec<Vec<[u8; 2]>> = Vec::new();

        for _index in 0..table[0] as usize {
            maximum_in_rows.push(0);
            current_in_rows_cells_location.push(Vec::new());
            maximum_in_rows_cells_location.push(Vec::new());
        }

        for index in 0..table[0] as usize {
            current_in_rows_cells_location[index].push([index as u8, 0]);
        }

        texts.
            get_mut(0).
            unwrap().
            get_mut(0).
            unwrap().
            text = graphics::Text::new(
                graphics::TextFragment::new(numbers.
                    get_mut(0).
                    unwrap().
                    get_mut(0).
                    unwrap().
                    to_string()
                ).
                font(graphics::Font::default()).
                color(graphics::Color::new(1.0, 0.0, 0.0, 1.0)).
                scale(graphics::PxScale::from(40.0))
            );

        menu_sum_texts.push(NumberText::new(
        graphics::Text::new(
            graphics::TextFragment::new(
                format!("{} + ", numbers.get(0).unwrap().get(0).unwrap().to_string())
            ).
            font(graphics::Font::default()).
            color(graphics::Color::new(1.0, 0.0, 0.0, 1.0)).
            scale(graphics::PxScale::from(15.0)))
        ));

        menu_sum_numbers.push(*numbers.get(0).unwrap().get(0).unwrap());

        Self {
            table,
            menu_background,
            menu_texts,
            menu_sum_texts,
            menu_max_text,
            menu_sum_numbers,
            menu_lines,
            cells,
            cell_width,
            cell_height,
            borders,
            numbers,
            texts,
            head_node,
            maximum_in_rows,
            current_in_rows_cells_location,
            maximum_in_rows_cells_location,
            current_milisec: wait_then_solve_in_miliseconds,
            next_milisec: 0,
            refresh_rate_in_miliseconds,
            game_not_finished
        }
    }

    fn update_logic(&mut self, context: &mut Context) {
        self.next_milisec = ggez::timer::time_since_start(context).as_millis() as u64;
        if self.current_milisec < self.next_milisec {
            self.current_milisec = self.next_milisec + self.refresh_rate_in_miliseconds;
            if self.game_not_finished {
                if self.head_node.game_not_finished {
                    let new_location = self.head_node.new_move(self.table, &mut self.numbers, &mut self.menu_sum_numbers, &mut self.current_in_rows_cells_location);
                    let mut color = graphics::Color::new(1.0, 0.0, 0.0, 1.0);
    
                    match self.head_node.start_row {
                        0 => {
                            color = graphics::Color::new(1.0, 0.0, 0.0, 1.0);
                        },
                        1 => {
                            color = graphics::Color::new(1.0, 0.5, 0.0, 1.0);
                        },
                        2 => {
                            color = graphics::Color::new(1.0, 1.0, 0.0, 1.0);
                        },
                        3 => {
                            color = graphics::Color::new(0.0, 1.0, 0.0, 1.0);
                        },
                        4 => {
                            color = graphics::Color::new(0.0, 0.0, 1.0, 1.0);
                        },
                        5 => {
                            color = graphics::Color::new(0.5, 0.0, 1.0, 1.0);
                        },
                        6 => {
                            color = graphics::Color::new(1.0, 0.0, 1.0, 1.0);
                        },
                        7 => {
                            color = graphics::Color::new(1.0, 1.0, 1.0, 1.0);
                        },
                        8 => {
                            color = graphics::Color::new(0.5, 0.5, 0.5, 1.0);
                        },
                        9 => {
                            color = graphics::Color::new(0.0, 0.0, 0.0, 1.0);
                        },
                        _ => {}
                    }
                    let mut maximum = String::new();
                    maximum.push_str("Max : ");
                    maximum.push_str(self.maximum_in_rows[self.head_node.start_row as usize].to_string().as_str());
                    maximum.push_str(" ->");
                    self.menu_texts.
                    get_mut(self.head_node.start_row as usize).
                    unwrap().
                    text = graphics::Text::new(
                        graphics::TextFragment::new(
                            maximum
                        ).
                        font(graphics::Font::default()).
                        color(color).
                        scale(graphics::PxScale::from(15.0))
                    );
                    self.texts.
                        get_mut(new_location[0] as usize).
                        unwrap().
                        get_mut(new_location[1] as usize).
                        unwrap().
                        text = graphics::Text::new(
                            graphics::TextFragment::new(self.numbers.
                                get_mut(new_location[0] as usize).
                                unwrap().
                                get_mut(new_location[1] as usize).
                                unwrap().
                                to_string()
                            ).
                            font(graphics::Font::default()).
                            color(graphics::Color::new(1.0, 0.0, 0.0, 1.0)).
                            scale(graphics::PxScale::from(40.0))
                        );
                    self.head_node.mesh = graphics::Mesh::new_circle(
                        context,
                        graphics::DrawMode::fill(),
                        Vector2D::new(576.0 + new_location[1] as f32 * self.cell_width + self.cell_width / 2.5, new_location[0] as f32 * self.cell_height + self.cell_height / 1.2),
                        3.0,
                        0.5,
                        color
                    ).unwrap();
    
                    self.menu_sum_texts.pop();
                    let mut new_menu_sum_text = String::new();
                    for number in self.menu_sum_numbers.iter() {
                        new_menu_sum_text.push_str(number.to_string().as_str());
                        new_menu_sum_text.push_str(" + ");
                    }
                    if self.menu_sum_numbers.len() == self.table[1] as usize {
                        new_menu_sum_text.remove(new_menu_sum_text.len() - 2);
                        new_menu_sum_text.push_str("= ");
                        let mut sum_for_menu_sum_numbers: u16 = 0;
    
                        for number in self.menu_sum_numbers.iter() {
                            sum_for_menu_sum_numbers += *number as u16;
                        }
                        let max = self.maximum_in_rows.get(self.head_node.start_row as usize).unwrap();
                        if sum_for_menu_sum_numbers > *max {
                            self.maximum_in_rows_cells_location[self.head_node.start_row as usize] = self.current_in_rows_cells_location[self.head_node.start_row as usize].clone();
                            self.maximum_in_rows[self.head_node.start_row as usize] = sum_for_menu_sum_numbers;
                        }
                        new_menu_sum_text.push_str(sum_for_menu_sum_numbers.to_string().as_str());
                    }
                    self.menu_sum_texts.push(NumberText::new(
                        graphics::Text::new(
                            graphics::TextFragment::new(
                            new_menu_sum_text
                        ).
                        font(graphics::Font::default()).
                        color(color).
                        scale(graphics::PxScale::from(15.0))
                    )));
                } else {
                    self.game_not_finished = false;
                    println!("Finished :D");

                    let mut max_of_max: u16 = 0;
                    let mut i: u8 = 0;
                    let mut j: u8 = 0;
                    for max in self.maximum_in_rows.iter() {
                        if *max > max_of_max {
                            max_of_max = *max;
                            j = i;
                        }
                        i += 1;
                    }
                    let mut final_max = String::new();
                    final_max.push_str("Final Maximum : ?");
                    final_max.push_str(max_of_max.to_string().as_str());
                    self.menu_max_text.text = graphics::Text::new(
                        graphics::TextFragment::new(
                            final_max
                        ).
                        font(graphics::Font::default()).
                        color(graphics::Color::new(0.0, 1.0, 0.0, 1.0)).
                        scale(graphics::PxScale::from(15.0))
                    );

                    let final_paths = self.maximum_in_rows_cells_location.get(j as usize).unwrap();
                    for final_path in final_paths.iter() {
                        self.texts.get_mut(final_path[0] as usize).unwrap().get_mut(final_path[1] as usize).unwrap().text = 
                            graphics::Text::new(
                                graphics::TextFragment::new(
                                    self.numbers.get(final_path[0] as usize).unwrap().get(final_path[1] as usize).unwrap().to_string()
                                ).
                                font(graphics::Font::default()).
                                color(graphics::Color::new(0.0, 1.0, 0.0, 1.0)).
                                scale(graphics::PxScale::from(40.0))
                            );
                    }
                }
            }
        }
    }

    fn update_graphic(&self, context: &mut Context) {
        graphics::draw(context, &self.menu_background, graphics::DrawParam::default()).expect("Error can't draw menu_background");

        let mut i: u8 = 0;
        for row in self.menu_texts.iter() {
            graphics::draw(context, &row.text, (Vector2D::new(20.0, 30.0 + i as f32 * self.cell_height), graphics::Color::WHITE)).expect("Error updating graphic");
            i += 1;
        }

        let mut i: u8 = 0;
        for row in self.menu_sum_texts.iter() {
            graphics::draw(context, &row.text, (Vector2D::new(120.0 + i as f32 * self.cell_width, 30.0 + self.head_node.start_row as f32 * self.cell_height), graphics::Color::WHITE)).expect("Error updating graphic");
            i += 1;
        }

        for row in self.menu_lines.iter() {
            graphics::draw(context, row, graphics::DrawParam::default()).expect("Error updating graphic");
        }

        for row in self.cells.iter() {
            for column in row.iter() {
                graphics::draw(context, &column.mesh, graphics::DrawParam::default()).expect("Error updating graphic");
            }
        }

        for row in self.borders.iter() {
            for column in row.iter() {
                graphics::draw(context, &column.mesh, graphics::DrawParam::default()).expect("Error updating graphic");
            }
        }

        i = 0;
        let mut j: u8 = 0;
        for row in self.texts.iter() {
            for column in row.iter() {
                graphics::draw(context, &column.text, (Vector2D::new(560.0 + j as f32 * self.cell_width + self.cell_width / 3.0, i as f32 * self.cell_height + self.cell_height / 4.0), graphics::Color::WHITE)).expect("Error updating graphic");
                j += 1;
            }
            j = 0;
            i += 1;
        }

        graphics::draw(context, &self.menu_max_text.text, (Vector2D::new(380.0, 680.0), graphics::Color::WHITE)).expect("Can't update final max graphic");

        graphics::draw(context, &self.head_node.mesh, graphics::DrawParam::default()).expect("Error can't draw head_node mesh");
    }
}

impl EventHandler for GoldMiner {
    fn update(&mut self, context: &mut Context) -> GameResult<()> {
        self.update_logic(context);
        Ok(())
    }
    
    fn draw(&mut self, context: &mut Context) -> GameResult<()> {
        graphics::clear(context, graphics::Color::WHITE);

        self.update_graphic(context);

        graphics::present(context)
    }
}