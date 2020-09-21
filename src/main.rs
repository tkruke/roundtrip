/*-------------------------------------
  RoundTrip

    Consider this puzzle
    The readers is presented with a board consisting of a n x m dot matrix and some lines connecting some of the dots.
    Can you find a way to connect all the dots with a closed loop, including the original lines?

    The original puzzle has 8x8 dots, but here we are going to consider the general case with n x m dots.
    If we connect all the dots with horizontal and vertical lines they form a  pattern of (n-1)x(m-1) square tiles.
    We call the dots 'vertices' and the lines 'edges'. Only horizontal and vertical edges are allowed.
    The problem: To connect ALL vertices so that the edges form a closed route traversing all vertices ONCE.

    If you play around with this a little bit you will soon discover that
     - each vertice must be connected with exactly two other vertices via exactly two edges
     - it follows that the number of edges = number of vertices = n * m
     - all corner vertices (top left/right and bottom left/right) have only two neighbours (no diagonal edges allowed)
        and must therefore have edges connecting them
     - there must be exactly n x m edges
     - the number of horizontal lines (edges) must be even, as must the number of vertical lines (edges)
     - from this it follows that n x m must be an even number and hence; n and/or m must be even
        (3x3, 5x5, 3x7 etc. has no solution - please feel free to try!)

    Other facts can be observed about the tiles when a valic closed loop has been established:
     - the number of tiles outside the loop = (n-2)*(m-2)/2
        (for example an 8 x 8 matrix will have (8-2)*(8-2)/2 = 18 tiles outside the loop )
     - it follows that the number of tiles inside the loop = (n-1)*(m-1) - (n-2)*(m-2)/2
        (for 8x8 matrix: (8-1)*(8-1) - 18 = 49-18 = 31 tiles will be inside the loop)
     - all corner tiles must be inside the loop
     - any 2x2 sub-matrix must have at least one tile outside the loop and one tile inside the loop
     - no 2x2 sub-matrix can have a checker-pattern with tiles inside and outside the loop
        (imagine coloring tiles inside the loop black and tiles outside the loop white)
     - two neighbour tiles at the rim of the matrix cannot both be outside the loop

    I originally came across this puzzle in a newspaper, and the reader was asked to complete the loop in a 8 x 8 matrix
    where a handfull of edges had already been established. The original state was given such that there was only one
    way to complete the loop according to the rules. After solving a few of these puzzles I started to consider some
    mathematical properties connected with the puzzle.

    First of all, I wondered how many different solutions could be created if we didn't put any edges into the start state.
    Then I started considering different matrix sizes. Is there a formula for the number of variations for an n x m matrix?
    It was pretty easy to prove that nxm had to be an even number. That is; there are no solutions to a n x m matrix when
    both n and m are odd numbers. Next, I explored small matrices by hand. It became obvious that the number of solutions
    grew very quickly as n and m indcreased. But with no obvious pattern to be seen from those small examples that I could
    calculate by hand. So I wrote a computer program to check for solutions for larger matrices.

    My first attempt was based on tilings of the (n-1) x (m-1) matrix. I soon found that the rule for solutions to the
    2 x m matrices formed the series 	1, 3, 4, 10, 16, 36, 64, 136, 256, 528, 1024, 2080, 4096... which I found in the
    on-line encyclopedia of integer sequences (https://oeis.org/) as the series A051437. However, none of the other
    dimensions seemed to give me any hits. This strikes me as strange, as I feel that this problem should follow some
    'known' combinatoric pattern.

    So here I set out to explore more. Maybe my first program had an error? Maybe I wasn't able to remove duplicates?
    Or maybe I should consider solutions that are identical when rotating the matrix as different? What about reflexive
    solutions - should they be counted as different or not? But most of all - was my first attempt, where I considered
    tiles inside vs outside the loop inferior to an attempt to find the edges?

    In this program we set out to search the solution space by traveling from an original vertice through edges to neighbouring
    edges until we find all possible versions ending with a valid solution (n * m edges in a closed loop through all vertices).
    I aim to find all possible paths, and then analyse the number of solutions that are similar under the opertion of
    rotation. If that doesn't give any clue for the pattern then I will consider also analyzing solutions that are similar under
    the operation of reflextion (both horizontal, vertical and around the two diagonals).

    Brønnøysund, 9.9.2020
    Torgeir Kruke

    v0.1 - project created and some initiating code in place

-------------------------------------*/

use std::io::{stdin, stdout, Write};
use std::time::SystemTime;

// adjust N_MAX and M_MAX equal to n and m if you want to optimize for memory usage (and possibly speed)
const N_MAX: usize = 20;
const M_MAX: usize = 20;

fn get_matrix_dimension() -> (usize, usize) {
    loop {
        let mut input_n = "".to_string();
        let mut input_m = "".to_string();
        println!("Enter matrix size N x M (or 0 to end)");
        print!("N: ");
        stdout().flush().ok().expect("Could not flush stdout");
        stdin()
            .read_line(&mut input_n)
            .expect("Could not read line");
        let n: usize = match input_n.trim().parse() {
            Ok(tall) => tall,
            Err(_) => {
                println!("Could not assign a value to n: {:?}", input_n);
                println!("Please try again");
                continue;
            }
        };
        print!("M: ");
        stdout().flush().ok().expect("Could not flush stdout");
        stdin()
            .read_line(&mut input_m)
            .expect("Could not read line");
        let m: usize = match input_m.trim().parse() {
            Ok(tall) => tall,
            Err(_) => {
                println!("Could not assign a value to m: {:?}", input_m);
                println!("Please try again");
                continue;
            }
        };
        return (n, m);
    }
}

fn validate_board_size(n: usize, m: usize) -> bool {
    if n > N_MAX || m > M_MAX {
        println!("n and m must be less or equal to {} and {}", N_MAX, M_MAX);
        return false;
    }
    if n > m {
        // enforce that N <= M (an NxM matrix have same solutions as a MxN matrix, so this is just to be able to assert a 'high and thin' matrix when checking and printing solutions)
        println!("n ({}) should be less or equal to m ({})", n, m);
        return false;
    }
    let size = n * m;
    if size < 12 {
        println!("Too small!");
        println!("Board size n*m must be min 12 and max 128.");
        return false;
    } else if size > 128 {
        println!("Too big!");
        println!("Board size n*m must be min 12 and max 128.");
        return false;
    } else if (n * m) & 1 == 1 {
        println!("Invalid matrix size");
        println!("n * m MUST be an even number");
        return false;
    } else {
        return true;
    }
}

fn initialize_board(
    mut b: [[bool; N_MAX * M_MAX]; N_MAX * M_MAX],
    n: usize,
    m: usize,
) -> [[bool; N_MAX * M_MAX]; N_MAX * M_MAX] {
    // Initialize the adjecency matrix
    // The diagonal b[i,i] will indicate if vertice i has been visited (1) or not (0)
    // For the rest of the matrix b[i][j] = 1 indicates that there is an (undirected) edge between vertices i and j
    // println!("... entering fn initialize_board");
    for j in 0..n * m {
        for i in 0..n * m {
            if (j >= n && (i == j-n))                          // i has j as its neighbour below
                || (j > 0 && (i == j-1 && ((i+1) % n != 0)))   // i has j as its neighbour to the right (if not i is at the right hand rim) 
                || (i == j+1 && (i % n != 0))       // i has j as its neighbour to the left (if not i is at the left hand rim)
                || (i == j+n)
            {
                // i has j as its neighbour above
                b[i][j] = true;
                // println!("b[{}][{}] = 1", i, j);
            }
        }
    }
    // println!("... returning from fn initialize_board");
    return b;
}

fn check_board(
    board: &mut Box<[[bool; N_MAX * M_MAX]; N_MAX * M_MAX]>,
    visited: usize,
    solutions: &mut i64,
    v: usize,
    n: usize,
    m: usize,
    systime: &SystemTime,
) {
    // parameters: board, #visited_so_far, #solutions_so_far, vertice_to_visit, matrix_dimension_n, matrix_dimension_m
    // println!("... entering fn check_board with (board = b, visited= {}, solutions = {}, vertice = {}, n={}, m={}", visited, solutions, v, n, m);
    // let mut input_str ="".to_string();
    // stdin().read_line(&mut input_str).expect("Could not read line");
    if visited + 1 == n * m {
        //all vertices visited - can we make it back to the start vertice (0)?
        if board[0][v] {
            // success!
            // println!("... SOLUTION found!");
            *solutions += 1;
            // println!("solution #{}!", solutions);
            if (*solutions + 1) % 1000 == 0 {
                println!("{:?}: {} solutions", systime.elapsed(), *solutions+1);
            }
            return;
        } else {
            // failure!
            return;
        }
    }

    board[v][v] = true; // mark vertice v as visited
    for i in 0..n * m {
        if i != v && board[i][v] && !board[i][i] {
            // (i==v is no edge) there is an edge from v to i, and vertice i has not been visited yet
            check_board(board, visited + 1, solutions, i, n, m, systime); // try finding a solution by traversing the edge from v to i and search for solutions from there
        }
    }
    board[v][v] = false; // mark vertice v as unvisited
    return;
}

fn main() {
    loop {
        println!("--- La Linea RoundTrip ---");
        let (n, m) = get_matrix_dimension();
        if n == 0 || m == 0 {
            break;
        }
        if validate_board_size(n, m) == false {
            println!("Adjust parameters and try again!");
        } else {
            /* All clear - GO! */
            println!("Initializing");
            let b = [[false; N_MAX * M_MAX]; N_MAX * M_MAX];
            let mut board = Box::new(initialize_board(b, n, m));
            println!("Searching solutions for {:?} x {:?} matrix", n, m);
            let run_duration = SystemTime::now();
            let mut solutions: i64 = 0;
            board[0][0] = true; // start top left in vertice 0
            let vertice_to_visit = 1; // define first step to the right to avoid checking solutions 'in both directions'
            check_board(
                &mut board,
                1,
                &mut solutions,
                vertice_to_visit,
                n,
                m,
                &run_duration,
            ); // parameters: board, #visited_so_far, #solutions_so_far, vertice_to_visit, matrix_dimension_n, matrix_dimension_m
            println!("{} solutions found", solutions);
            println!("Run duration: {:?}", run_duration.elapsed());
        }
    }
}
