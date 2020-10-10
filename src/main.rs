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
    It is pretty easy to prove that nxm has to be an even number. That is; there are no solutions to a n x m matrix when
    both n and m are odd. Next, I explored small matrices by hand. It became obvious that the number of solutions
    grew very quickly as n and m increased. But I was not able to see any obvious pattern  from the small examples I could
    calculate by hand. So I wrote a computer program to check for solutions for larger matrices.

    My first attempt was based on tilings of the (n-1) x (m-1) matrix. I soon found that the rule for solutions to the
    2 x m matrices formed the series 	1, 3, 4, 10, 16, 36, 64, 136, 256, 528, 1024, 2080, 4096... which I found in the
    on-line encyclopedia of integer sequences (https://oeis.org/) as the series A051437. However, none of the other
    dimensions seemed to give me any hits. This strikes me as strange, as I feel that this problem should follow some
    'known' combinatoric pattern.

    So here I set out to explore more. Maybe my first program had an error? Maybe I wasn't able to remove duplicates?
    Or maybe I should consider remove solutions that are identical when rotating the matrix? What about 'flipped'
    solutions - should they be counted as different or not? But most of all - was my first attempt not really equivalent
    to the original problem of finding the edges that form a closed loop including all vertices?

    In this program we set out to search the solution space by traveling from an original vertice through edges to neighbouring
    edges until we find all possible versions ending with a valid solution (n*m edges in a closed loop visiting all vertices).
    The algorithm aims to find all possible paths. And this time we don't care if we get patterns that are 'equal' when rotated. 
    
    Brønnøysund, 9.9.2020
    Torgeir Kruke

    v0.1 - project created and some initiating code in place
    v0.2 - first version generating solutions for matrices up to about 6x6
    v0.3 - using box'ing to avoid stack usage (thanks Anders :)


-------------------------------------*/

use std::io::{stdin, stdout, Write};
use std::time::SystemTime;

// adjust N_MAX and M_MAX equal to n and m if you want to optimize for memory usage (and possibly speed)
const N_MAX: usize = 20;
const M_MAX: usize = 20;

struct Metrics {
    pub run_duration: SystemTime,
    pub check_counter: i64,
    pub fail_counter_1: i64,
    pub fail_counter_2: i64,
    pub fail_counter_3: i64,
    pub exception_counter: i64,
    pub solutions_counter: i64,
    pub visited_vertices: usize,
    pub visited_rim_vertices: usize,
}

impl Metrics {
    pub fn new () -> Self {
        Metrics {
            run_duration: SystemTime::now(),
            check_counter: 0,
            fail_counter_1: 0,
            fail_counter_2: 0,
            fail_counter_3: 0,
            exception_counter: 0,
            solutions_counter: 0,
            visited_vertices: 0,
            visited_rim_vertices: 0,
        }
    }
}

fn get_matrix_dimension() -> (usize, usize) {
    loop {
        let mut input_n = "".to_string();
        let mut input_m = "".to_string();
        println!("Enter matrix size n x m (or 0 to end)");
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

fn initialize_board(board: &mut [[bool; N_MAX * M_MAX]; N_MAX * M_MAX], rim_vertices: &mut Vec<usize>, n: usize, m: usize, )  {
    // Initialize the adjecency matrix
    // The diagonal b[i,i] will indicate if vertice i has been visited (true) or not (false)
    // For the rest of the matrix b[i][j] = true indicates that there is an edge from vertice i to j
    // If there is an edge both from i to j and from j to i, the edge betweed i and j is undirected
    for j in 0..n * m {
        for i in 0..n * m {
            // println!("i, j = {}, {}", i, j);
            if (j >= n && (i == j-n))                        // i has j as its neighbour below
                || (j > 0 && (i == j-1 && ((i+1) % n != 0))) // i has j as its neighbour to the right (check that i is not at the right rim) 
                || (i == j+1 && (i % n != 0))                // i has j as its neighbour to the left (check that i is not at the left hand rim)
                || (i == j+n)                                // i has j as its neighbour above
            {
                board[i][j] = true;
                // b[j][i] = true;  
                    // this is surperflous since we traverse all nodes and include all valid neighbours; 
                    // hence j -> i will also always be covered
            }
        }
    }

    // Now enforce some special initialization of directed edges for the rim nodes:
    //      (1) Edges can only be traveled from a rim node to the inside of the lattice (initially)
    //              (we reverse this operation for one edge at a time - whenever we travel from a rim node to the interior of the lattice
    //              we allow the path to return to the rim through the (clockwise) neighbour edge)
    //  AND (2) Nodes at the rim can only be visited in a clockwise direction 
    //              (hence no anti-clockwise edges between rim nodes)
    //  Note: (2) enforces that we only find paths going in clockwise direction. Otherwise we would find all paths twice; once 
    //        going in a clockwise direction and once going anti-clockwise. 
    for i in 0..n-1 {
        println!("i = {}", i);
        board[i+n+1][i+1] = false;                  // from second row to top (first) row
        board[n*(m-2)+i][n*(m-1)+i] = false;        // from second to last row to bottom (last) row
        board[i+1][i] = false;                      // top row
        board[n*(m-1)+i][n*(m-1)+i+1] = false;      // bottom row
    }
    for j in 0..m-1 {
        println!("j = {}", j);
        board[j*n+1][j*n] = false;                  // from second column to left rim (first) column
        board[(j+2)*n - 2][(j+2)*n - 1] = false;    // from second to last column to right rim (last) column
        board[j*n][(j+1)*n] = false;                // left rim column
        board[(j+2)*n-1][(j+1)*n-1] = false;        // right rim column
    }

    // Make a vector with all rim nodes in clockwise direction sequence (there are 2n+2m-4 rim nodes)
    for i in 0..n {
        rim_vertices.push(i);
    }
    for j in 1..m {
        rim_vertices.push((j+1)*n - 1);
    }
    for i in 1..n {
        rim_vertices.push(n*m-1 - i);
    } 
    for j in 1..m-1 {
        rim_vertices.push(n*(m-1) - j*n); 
    }
    println!("#rim vertices = {}", rim_vertices.len()+1);
    for i in 0..rim_vertices.len() {
        println!("rim_vertices[{}] = {}", i, rim_vertices[i]);
    }

    // println!("... finished initializing board");
    return;
}

fn check_board(board: &mut [[bool; N_MAX * M_MAX]; N_MAX * M_MAX], 
    rim_vertices: &Vec<usize>, 
    // mut visited_rim_vertices: usize, 
    // visited_vertices: usize, 
    solution_path: &mut Vec<usize>, 
    // solutions_counter: &mut i64, 
    v: usize, 
    n: usize, 
    m: usize, 
    metrics: &mut Metrics,)
    // check_counter: &mut i64, 
    // fail_counter_1: &mut i64,
    // fail_counter_2: &mut i64,
    // fail_counter_3: &mut i64,
    // exception_counter: &mut i64,
    // run_duration: &SystemTime) 
    {
    metrics.check_counter += 1;
    // print!("{},", v);  // debug print
    let at_the_rim = rim_vertices.contains(&v); 
    if at_the_rim {
        metrics.visited_rim_vertices += 1;
        if metrics.visited_rim_vertices == rim_vertices.len() && metrics.visited_vertices + 1 < n*m {
            metrics.fail_counter_2 += 1;
            //println!("no more rim - backtrack, check_counter = {}", check_counter);  // debug print
            //print!("-{},", v); // debug print
            return;  // all rim vertices has been visited, but there remains unvisited interior vertices => fail!
        } 
    }
    if metrics.visited_vertices + 1 == n * m {
        //all vertices visited - can we make it back to the start vertice (0)?
        // ToDo: Prove that if you get here you MUST have a solution - so, no need for a final check
        //if board[v][0] {
            // success!
            // println!("... SOLUTION found!");
            metrics.solutions_counter += 1;
            //println!("solution #{}!", solutions_counter);
            //print!("-{}", v);
            if (metrics.solutions_counter + 1) % 10000 == 0 {
                println!("{:?}: {} solutions", metrics.run_duration.elapsed(), metrics.solutions_counter+1);
            }
            return;
        //} else {
            // failure!
        //    *fail_counter_1 += 1;
            //println!("fail, check_counter = {}", check_counter);
            //print!("-{}", v);
        //    return;
        //}
    }

    board[v][v] = true; // mark vertice v as visited
    solution_path.push(v);
    let mut store_j = 0;
    let mut store_next_rim_vertice = 0;
    for i in 0..n * m {
        if board[v][i] && !board[i][i] {
            // there is an edge from v to i, and vertice i has not been visited yet
            let mut check_i = true;
            if at_the_rim {
                if !rim_vertices.contains(&i) { // we are about to enter the interior of the lattice...
                    check_i = false;            // ... but we don't want to go to the interior unless we can set a return path to the rim
                    // (this will happen when we are next to a corner vertice, or if we already visited the vertice providing the return edge)
                    let next_rim_vertice = rim_vertices[metrics.visited_rim_vertices];
                    for j in 0..n*m {
                        if board[next_rim_vertice][j] && !rim_vertices.contains(&j) && !board[j][j] {
                            board[j][next_rim_vertice] = true;  // we 'open' the return edge from the interior to the next rim vertice
                            store_j = j; 
                            store_next_rim_vertice = next_rim_vertice;
                            check_i = true;  // return edge found - ok to continue 
                            continue;
                        }
                    }
                }
            } else if !rim_vertices.contains(&i) {  // if we stay interior to the lattice...
                //... then we can abort if we are about to create two separate 'islands' of unvisited vertices.
                //  I.e. if we have unvisited vertices both left and right, while at least one of the vertices in front has been visited already.
                //  This logic can be extended to incorporate cases when two regions are only connected through a single track
                //  If so, we can only complete a cycle if we are in the opposite region to the one where our endpoint is.
                //  In the special case where we approach the rim, the rim will act as such a single track connection and we have to go
                //  to the left since the endpoint will always be to the right.
                if v>i && v-i == n {
                    // direction = 'n';
                    if !board[v+n][v+n] && rim_vertices.contains(&(v-1)) && !board[v-1][v-1] {
                        check_i = false;
                    }   // "must go left" => cannot go this way
                    if (board[i-n][i-n] || board[i-n-1][i-n-1] || board [i-n+1][i-n+1]) &&
                        (!board[i-1][i-1] && !board[i+1][i+1]) {
                            check_i = false;
                    }
                } else if v<i && i-v == n {
                    // direction = 's';
                    if !board[v-n][v-n] && rim_vertices.contains(&(v+1)) && !board[v+1][v+1] {
                        check_i = false;
                        //println!("south - must go left=north");
                        //println!("v={}, i={}, board[v-n][v-n]={}, {}, solution_path:", v, i, board[v-n][v-n], rim_vertices.contains(&(v+1)));
                        //for k in 0..solution_path.len() {
                        //    print!("{},", solution_path[k]);
                        //}
                        //println!("");
                    }   // "must go left" => cannot go this way
                    if (board[i+n][i+n] || board[i+n-1][i+n-1] || board[i+n+1][i+n+1]) &&
                        (!board[i-1][i-1] && !board[i+1][i+1]) {
                            check_i = false;
                    } 
                } else if v>i && v-i == 1 {
                    // direction = 'w';
                    if !board[v+1][v+1] && rim_vertices.contains(&(v+n)) && !board[v+n][v+n] {
                        check_i = false;
                    }   // "must go left" => cannot go this way
                    if (board[i-1][i-1] || board[i-1+n][i-1+n] || board[i-1-n][i-1-n]) &&
                        (!board[i+n][i+n] && !board[i-n][i-n]) {
                            check_i = false;
                    }
                } else if v<i && i-v == 1 {
                    // direction = 'e';
                    // If direction is 'east' we will always be going 'left'
                    if (board[i+1][i+1] || board[i+1+n][i+1+n] || board[i+1-n][i+1-n]) &&
                        (!board[i+n][i+n] && !board[i-n][i-n]) {
                            check_i = false;
                    }
                }

            }
            if check_i {
                // traverse edge from v to i and search for solutions from there
                metrics.visited_vertices += 1;
                check_board(board, 
                    rim_vertices, 
                    // visited_rim_vertices, 
                    // visited_vertices + 1, 
                    solution_path, 
                    // solutions_counter, 
                    i, // next vertice to visit
                    n, m, 
                    metrics, 
                    // check_counter, fail_counter_1, fail_counter_2, fail_counter_3, exception_counter, run_duration
                ); 
                metrics.visited_vertices -= 1;
            }
        }
    }
    board[v][v] = false; // mark vertice v as unvisited
    solution_path.pop();
    board[store_j][store_next_rim_vertice] = false; // reset if 'return edge' was set true 
    // println!("backtrack - check_counter = {}", check_counter);
    //print!("-{},", v);  // debug print
    metrics.fail_counter_3 += 1;
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
            println!("Initializing");
            let mut board = [[false; N_MAX * M_MAX]; N_MAX * M_MAX];
            let mut rim_vertices: Vec<usize> = vec![];
            initialize_board(&mut board, &mut rim_vertices, n, m);
            println!("Searching solutions for {:?} x {:?} matrix", n, m);
            let mut metrics = Metrics::new();
            // let run_duration = SystemTime::now();
            // let mut check_counter: i64 = 0;
            // let mut fail_counter_1: i64 = 0;
            // let mut fail_counter_2: i64 = 0;
            // let mut fail_counter_3: i64 = 0;
            // let mut exception_counter: i64 = 0;
            // let mut solutions_counter: i64 = 0;
            // let visited_vertices = 0;
            // let visited_rim_vertices = 0;
            let vertice_to_visit = 0;   // start with vertice 0
            let mut solution_path: Vec<usize> = vec![];
            check_board(
                &mut board,
                &rim_vertices,
                // visited_rim_vertices,
                // visited_vertices,
                &mut solution_path, 
                // &mut solutions_counter, 
                vertice_to_visit, 
                n,
                m,
                &mut metrics,
                // &mut check_counter,
                // &mut fail_counter_1,
                // &mut fail_counter_2,
                // &mut fail_counter_3,
                // &mut exception_counter,
                // &run_duration,
            );
            println!("");
            println!("{} solutions found", metrics.solutions_counter);
            println!("Check_counter = {}", metrics.check_counter);
            println!("Fail counter 1 = {}", metrics.fail_counter_1);
            println!("Fail counter 2 = {}", metrics.fail_counter_2);
            println!("Fail counter 3 = {}", metrics.fail_counter_3);
            println!("Exception counter = {}", metrics.exception_counter);
            println!("Run duration: {:?}", metrics.run_duration.elapsed());
        }
    }
}
