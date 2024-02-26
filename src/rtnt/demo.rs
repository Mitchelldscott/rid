// use libm::{exp, floorf, sin, sqrtf};





// pub struct RTSinusiod {
//     amplitude: f32,
//     frequency: f32,
//     phase_lag: f32,
// }

// impl RTSinusiod {
//     pub fn new(data: RTTaskOutput) -> RTSinusiod {

//         RTSinusiod {
//             amplitude: f32::from_be_bytes([
//                     data[0],
//                     data[1],
//                     data[2],
//                     data[3],
//                 ]),
//             frequency: f32::from_be_bytes([
//                     data[4],
//                     data[5],
//                     data[6],
//                     data[7],
//                 ]),
//             phase_lag: f32::from_be_bytes([
//                     data[8],
//                     data[9],
//                     data[10],
//                     data[11],
//                 ]),
//         }

//     }

//     pub fn run(&self, t: f32) -> Option<f32> {

//         Some(self.amplitude * sin((self.frequency * t) + self.phase_lag))
    
//     }
// }

// pub struct RTSquarewave {
//     pub amplitude: f32,
//     pub duty_cycle: u8,
//     pub counter: u8,
// }

// impl RTSquarewave {
//     pub fn new(data: RTTaskOutput) -> RTSquarewave {
//         RTSquarewave {
//             amplitude: f32::from_be_bytes([
//                     data[4],
//                     data[5],
//                     data[6],
//                     data[7],
//                 ]),
//             duty_cycle: u32::from_be_bytes([
//                     data[0],
//                     data[1],
//                     data[2],
//                     data[3],
//                 ]),
//             counter: 0,
            
//         }
//     }

//     pub fn run(&self, t: f32) -> Option<f32> {

//         self.counter += 1;

//         match self.counter < self.duty_cycle {
//             true => Some(amplitude),
//             false => {

//                 self.counter = self.counter % 100;

//                 Some(0)
//             }
//         }
//     }
// }

// pub struct RTStateSpace {
//     pub n: u8,
//     pub m: u8,
//     pub k: u8,
//     pub x: [f32; 6],
//     pub A: [[f32; 6]; 6],
//     pub B: [[f32; 6]; 6],
//     pub C: [[f32; 6]; 3],
//     pub D: [[f32; 6]; 3],
// }

// impl RTStateSpace {
//     pub fn new(n: u8, m: u8, k: u8, x: [f32; 6]) -> RTStateSpace {
//         RTStateSpace {
//             n: n,
//             m: m,
//             k: k
//             x: x,
//             A: [[0.0f32; 6]; 6],
//             B: [[0.0f32; 6]; 6],
//             C: [[0.0f32; 6]; 6],
//             D: [[0.0f32; 6]; 6],
//         }
//     }

//     pub fn run(&self, input: [f32; 6]) -> Option<[f32; 6]> {
        
//         let mut dx = [0.0f32; self.n];
//         let mut y = [0.0f32; 6];

//         for n in 0..self.n {
//             for ni in 0..self.n {
//                 dx[n] += self.A[n][ni] * self.x[ni];
//             }

//             for m in 0..self.m {
//                 dx[n] += self.B[n][m] * input[m];
//             }
//         }

//         for k in 0..self.k {
//             for n in 0..self.n {
//                 y[k] += self.C[k][n] * (self.x[n] + dx[n]);
//             }

//             for m in 0..self.m {
//                 y[k] += self.D[k][m] * input[m];
//             }
//         }

//         // implement integrators for fancy stuff
//         for n in 0..self.n {
//             self.x[n] += dx[n];
//         }

//         Some(y)
//     }
// }

// pub struct RTPolynomial {
//     pub exp_map1: [u8; 16],
//     pub exp_map2: [u8; 16],
//     pub exp_map3: [u8; 16],
//     pub coeffs: [f32; 16],
// }

// impl RTPolynomial {
//     pub fn new(exp_map1: [u8; 16], exp_map2: [u8; 16], exp_map3: [u8; 16], coeffs: [f32; 16]) -> RTPolynomial {
//         RTPolynomial {
//             exp_map1,
//             exp_map2,
//             exp_map3,
//             coeffs
//         }
//     }

//     pub fn run(&self, input1: f32, input2: f32, input3: f32) -> Option<f32> {

//         let mut output = 0.0f32;

//         for i in (0..16) {

//             let mut expanded_input = 1.0f32;

//             for _ in (0..self.exp_map1[i]) {
//                 expanded_input[i] *= input1;
//             }

//             for _ in (0..self.exp_map2[i]) {
//                 expanded_input[i] *= input2;
//             }

//             for _ in (0..self.exp_map3[i]) {
//                 expanded_input[i] *= input3;
//             }

//             output += self.coeff[i] * expanded_input;

//         }

//         Some(output)
//     }
// }