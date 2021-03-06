/**
 * Flow - Realtime log analyzer
 * Copyright (C) 2016 Daniel Mircea
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::ops::Index;

use ncurses::wmove;

use core::line::Line;
use ui::content::Content;
use ui::frame::NORMAL_HIGHLIGHT_COLOR;
use ui::printer::{Print, Viewport};
use ui::highlighter::LineHighlighter;

#[derive(Clone)]
pub struct RenderedLine {
    pub line: Line,
    pub height: i32,
    pub found_matches: Option<Vec<usize>>,
}

impl RenderedLine {
    fn new(line: Line, height: i32, found_matches: Option<Vec<usize>>) -> RenderedLine {
        RenderedLine {
            line: line,
            height: height,
            found_matches: found_matches,
        }
    }

    pub fn search(&mut self,
                  text: &str,
                  content: &Content,
                  container_width: i32,
                  accumulated_height: i32)
                  -> bool {
        let is_match = self.line.contains(text);
        let mut found_matches = None;

        if is_match {
            self.print(content, accumulated_height);
            found_matches = self.highlight(text, content, container_width, accumulated_height);
        }

        if self.update_found_matches(found_matches) && !is_match {
            self.print(content, accumulated_height);
        }

        is_match
    }

    pub fn highlight(&self,
                     text: &str,
                     content: &Content,
                     container_width: i32,
                     accumulated_height: i32)
                     -> Option<Vec<usize>> {
        let highlighter = LineHighlighter::new(content.window,
                                               &self.line,
                                               container_width,
                                               NORMAL_HIGHLIGHT_COLOR);
        Some(highlighter.print(text, accumulated_height, self.height))
    }

    pub fn print(&self, content: &Content, accumulated_height: i32) {
        wmove(content.window, accumulated_height, 0);
        self.line.print(content);
    }

    pub fn update_found_matches(&mut self, found_matches: Option<Vec<usize>>) -> bool {
        if self.found_matches != found_matches {
            self.found_matches = found_matches;

            true
        } else {
            false
        }
    }

    pub fn match_count(&self) -> usize {
        self.found_matches.as_ref().unwrap().len()
    }
}

#[derive(Clone)]
pub struct RenderedLineCollection {
    pub entries: Vec<RenderedLine>,
}

impl RenderedLineCollection {
    pub fn default() -> RenderedLineCollection {
        RenderedLineCollection { entries: vec![] }
    }

    pub fn create(&mut self, line: Line, height: i32, found_matches: Option<Vec<usize>>) {
        let entry = RenderedLine::new(line, height, found_matches);
        self.entries.push(entry);
    }

    pub fn matching(&mut self, text: &str) -> RenderedLineCollection {
        RenderedLineCollection {
            entries: self.entries
                .iter()
                .filter(|entry| entry.line.contains(text))
                .map(|entry| entry.clone())
                .collect::<Vec<_>>(),
        }
    }

    pub fn height(&self) -> i32 {
        self.entries.iter().height()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn buffer_reverse_index(&self, line_index: usize, match_index: usize) -> i32 {
        let offset = self.entries[line_index].found_matches.as_ref().unwrap()[match_index];
        self.entries.iter().skip(line_index).height() - offset as i32
    }

    pub fn height_up_to_index(&self, index: usize) -> i32 {
        self.entries.iter().take(index).height()
    }

    pub fn last_lines_height(&self, count: usize) -> i32 {
        self.entries.iter().rev().take(count).height()
    }

    pub fn is_match_in_viewport(&self, matched_line: MatchedLine, viewport: Viewport) -> bool {
        let limit = viewport.limit();
        let accumulated_height = self.entries.iter().skip(matched_line.line).height() as usize;
        let line = &self.entries[matched_line.line];

        if accumulated_height >= viewport.reverse_index {
            for height in line.found_matches.as_ref().unwrap().iter() {
                if accumulated_height + height <= limit {
                    return true;
                }
            }
        }

        false
    }

    pub fn viewport_match(&self, viewport: &Viewport) -> Option<MatchedLine> {
        let mut accumulated_height = 0;
        let limit = viewport.limit();

        for (i, line) in self.entries.iter().rev().enumerate() {
            if accumulated_height >= viewport.reverse_index && line.found_matches.is_some() {
                for (j, height) in line.found_matches.as_ref().unwrap().iter().enumerate() {
                    if accumulated_height + height <= limit {
                        return Some(MatchedLine::new(i, j));
                    }
                }
            }

            accumulated_height += line.height as usize;

            if accumulated_height >= limit {
                break;
            }
        }

        None
    }

    pub fn last_match(&self) -> MatchedLine {
        self.entries
            .iter()
            .rev()
            .enumerate()
            .find_match()
            .unwrap()
    }

    pub fn next_match(&self, current_index: usize) -> Option<MatchedLine> {
        self.entries
            .iter()
            .enumerate()
            .skip(current_index + 1)
            .find_match()
    }

    pub fn previous_match(&self, current_index: usize) -> Option<MatchedLine> {
        self.entries
            .iter()
            .enumerate()
            .rev()
            .skip(self.entries.len() - current_index)
            .find_match()
    }
}

trait Height {
    fn height(self) -> i32;
}

impl<'a, I> Height for I
    where I: Iterator<Item = &'a RenderedLine>
{
    fn height(self) -> i32 {
        self.fold(0, |sum, current| sum + current.height)
    }
}

pub struct MatchedLine {
    pub line: usize,
    pub match_index: usize,
}

impl MatchedLine {
    pub fn new(line: usize, match_index: usize) -> MatchedLine {
        MatchedLine {
            line: line,
            match_index: match_index,
        }
    }
}

trait FindMatch {
    fn find_match(&mut self) -> Option<MatchedLine>;
}

impl<'a, I> FindMatch for I
    where I: Iterator<Item = (usize, &'a RenderedLine)>
{
    fn find_match(&mut self) -> Option<MatchedLine> {
        if let Some(value) = self.find(|&idx_and_line| idx_and_line.1.found_matches.is_some()) {
            Some(MatchedLine::new(value.0, value.1.match_count() - 1))
        } else {
            None
        }
    }
}

impl Index<usize> for RenderedLineCollection {
    type Output = RenderedLine;

    fn index(&self, _index: usize) -> &RenderedLine {
        &self.entries[_index]
    }
}
