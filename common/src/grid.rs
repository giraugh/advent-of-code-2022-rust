trait Grid<T> {
    /// Get a reference to the value in a cell
    fn get(&self, x: usize, y: usize) -> Option<&T>;

    /// Get a mutable reference to a cell
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T>;

    /// Get whether a cell location is in bounds
    fn in_bounds(&self, x: usize, y: usize) -> bool;

    /// The number of columns in the grid
    fn width(&self) -> usize;

    /// The number of rows in the grid
    fn height(&self) -> usize;

    /// Consume the grid to get a vector of every cell value
    fn cells(self) -> Vec<T>;

    /// Total number of cells
    fn count(&self) -> usize {
        self.width() * self.height()
    }

    /// Iterate over cell value references, row by row
    fn iter_rows(&self) -> GridIterator<T, Self>
    where
        Self: std::marker::Sized,
    {
        GridIterator {
            grid: self,
            x: 0,
            y: 0,
            by_rows: true,
            marker: std::marker::PhantomData,
        }
    }

    /// Iterate over cell value references, column by column
    fn iter_cols(&self) -> GridIterator<T, Self>
    where
        Self: std::marker::Sized,
    {
        GridIterator {
            grid: self,
            x: 0,
            y: 0,
            by_rows: false,
            marker: std::marker::PhantomData,
        }
    }
}

struct VecGrid<T> {
    cells: Vec<T>,
    width: usize,
    height: usize,
}

#[allow(dead_code)]
impl<T> VecGrid<T> {
    pub fn new(width: usize, height: usize) -> Self
    where
        T: Clone + Default,
    {
        Self {
            cells: vec![Default::default(); width * height],
            width,
            height,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        self.width * y + x
    }
}

impl<T> Grid<T> for VecGrid<T> {
    fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.in_bounds(x, y).then(|| &self.cells[self.index(x, y)])
    }

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let index = self.index(x, y);
        self.in_bounds(x, y).then(|| &mut self.cells[index])
    }

    fn in_bounds(&self, x: usize, y: usize) -> bool {
        x > 0 && x < self.width && y > 0 && y < self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn count(&self) -> usize {
        self.cells.len()
    }

    fn cells(self) -> Vec<T> {
        self.cells
    }
}

struct GridIterator<'a, T, G>
where
    G: Grid<T>,
{
    marker: std::marker::PhantomData<T>,
    grid: &'a G,
    x: usize,
    y: usize,
    by_rows: bool,
}

impl<'a, T: 'a, G: Grid<T>> Iterator for GridIterator<'a, T, G> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // Move in grid
        if !self.by_rows {
            self.x += 1;
            if self.x > self.grid.width() {
                self.x = 0;
                self.y += 1;
            }
        } else {
            self.y += 1;
            if self.y > self.grid.height() {
                self.y = 0;
                self.x += 1;
            }
        }

        // Return current item if applicable
        self.grid.get(self.x, self.y)
    }
}

struct VecGridTripleIterator<T> {
    grid_width: usize,
    cells: Vec<T>,
    offset: usize,
}

impl<T> VecGridTripleIterator<T> {
    fn new(grid: VecGrid<T>) -> Self {
        Self {
            grid_width: grid.width(),
            cells: grid.cells,
            offset: 0,
        }
    }
}

impl<T> Iterator for VecGridTripleIterator<T> {
    type Item = (usize, usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.offset += 1;
        self.cells.pop().map(|value| {
            let x = self.offset % self.grid_width;
            let y = self.offset / self.grid_width;
            (x, y, value)
        })
    }
}

impl<T> IntoIterator for VecGrid<T> {
    type Item = (usize, usize, T);
    type IntoIter = VecGridTripleIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        VecGridTripleIterator::new(self)
    }
}
