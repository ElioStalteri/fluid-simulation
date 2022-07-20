const N:i32 = 100;
const iter:i32 = 1;
// const SCALE:i32 = 4;

fn constrain(x:i32,min:i32,max:i32) ->i32{
    if x < min {
        return min;
    }else if x>max{
        return max;
    }
    return x;
}

fn IX( x_:i32, y_:i32) -> usize {
  let x = constrain(x_, 0, N-1);
  let y = constrain(y_, 0, N-1);
  return (x + (y * N)) as usize;
}

pub struct Fluid {
  size:i32,
  dt:f64, //time step
  diff:f64, //diffusion amount
  visc:f64, //thickness of fluid

  s:Vec<f64>, //previous density
  pub density:Vec<f64>,

  Vx:Vec<f64>,
  Vy:Vec<f64>,

  Vx0:Vec<f64>,
  Vy0:Vec<f64>,

}
impl Fluid{
  pub fn create(dt:f64, diffusion:f64,viscosity:f64) -> Fluid {
    return Fluid{
      size:N,
      dt : dt,
      diff : diffusion,
      visc : viscosity,
      s : vec![0f64;(N*N) as usize],
      density : vec![0f64;(N*N) as usize],
      Vx : vec![0f64;(N*N) as usize],
      Vy : vec![0f64;(N*N) as usize],
      Vx0 : vec![0f64;(N*N) as usize],
      Vy0 : vec![0f64;(N*N) as usize],
    }
  }

  pub fn step(&mut self) {
    Fluid::diffuse(1, &mut self.Vx0, &mut self.Vx, self.visc, self.dt);
    Fluid::diffuse(2, &mut self.Vy0, &mut self.Vy, self.visc, self.dt);
    
    Fluid::project(&mut self.Vx0, &mut self.Vy0, &mut self.Vx, &mut self.Vy);

    let Vx0 = self.Vx0.clone(); 
    let Vy0 = self.Vy0.clone();
    Fluid::advect(1, &mut self.Vx, &mut self.Vx0, Vx0, Vy0, self.dt);
    let Vx0 = self.Vx0.clone(); 
    let Vy0 = self.Vy0.clone();
    Fluid::advect(2, &mut self.Vy, &mut self.Vy0, Vx0, Vy0, self.dt);

    Fluid::project(&mut self.Vx, &mut self.Vy, &mut self.Vx0, &mut self.Vy0);
    
    Fluid::diffuse(0, &mut self.s, &mut self.density, self.diff, self.dt);
    let Vx = self.Vx.clone(); 
    let Vy = self.Vy.clone();
    Fluid::advect(0, &mut self.density, &mut self.s, Vx, Vy, self.dt);
 
  }

  pub fn addDensity(&mut self, x:i32,y:i32, amount:f64) {
    let index = IX(x, y);
    self.density[index] += amount;
  }


  pub fn addVelocity(&mut self,x:i32,  y:i32, amountX:f64,amountY:f64) {
    let index = IX(x, y);
    self.Vx[index] += amountX;
    self.Vy[index] += amountY;
  }




  fn diffuse(b: i32, x: &mut Vec<f64>, x0: &mut Vec<f64>, diff: f64, dt: f64) {
    let a: f64 = dt * diff * ((N - 2) * (N - 2) ) as f64;
    Fluid::lin_solve(b, x, x0, a, 1f64 + 6f64 * a);
  }

  fn lin_solve(b: i32, x: &mut Vec<f64>, x0: &mut Vec<f64>, a: f64, c: f64) {
    let cRecip = 1.0 / c;
    for k in 0..iter {
      for j in 1..(N - 1) {
        for i in 1..(N - 1) {
          x[IX(i, j)] =
            (x0[IX(i, j)]
            + a*(    x[IX(i+1, j)]
            + x[IX(i-1, j)]
            + x[IX(i, j+1)]
            + x[IX(i, j-1)]
            )) * cRecip;
        }
      }
      //Fluid::set_bnd(b, x);
    }
  }

  fn project(velocX: &mut Vec<f64>, velocY: &mut Vec<f64>, p: &mut Vec<f64>, div: &mut Vec<f64>) {
    for j in 1..(N - 1) {
      for i in 1..(N - 1) {
        div[IX(i, j)] = -0.5f64*(
          (velocX[IX(i+1, j)]-velocX[IX(i-1, j)])/(N as f64) + 
          (velocY[IX(i, j+1)]-velocY[IX(i, j-1)])/(N as f64)
          );
        p[IX(i, j)] = 0f64;
      }
    }
    Fluid::set_bnd(0, div); 
    Fluid::set_bnd(0, p);
    Fluid::lin_solve(0, p, div, 1f64, 6f64);

    for j in 1..(N - 1) {
      for i in 1..(N - 1) {
        velocX[IX(i, j)] -= 0.5f64 * (   p[IX(i+1, j)]
          - p[IX(i-1, j)]) * (N as f64);
        velocY[IX(i, j)] -= 0.5f64 * (   p[IX(i, j+1)]
          - p[IX(i, j-1)]) * (N as f64);
      }
    }
    Fluid::set_bnd(1, velocX);
    Fluid::set_bnd(2, velocY);
  }


  fn advect(b: i32, d: &mut Vec<f64>, d0: &mut Vec<f64>, velocX: Vec<f64>, velocY: Vec<f64>, dt: f64)
  {
    // float i0, i1, j0, j1;

    let dtx = dt * ((N - 2) as f64);
    let dty = dt * ((N - 2) as f64);

    // float s0, s1, t0, t1;
    // float tmp1, tmp2, x, y;

    let Nfloat = N as f64;
    
    // float ifloat, jfloat;
    // int i, j;

    for j in 1..(N - 1) { 
      for i in 1..(N - 1) {
        let jfloat = j as f64;
        let ifloat = i as f64;
        let tmp1 = dtx * velocX[IX(i, j)];
        let tmp2 = dty * velocY[IX(i, j)];
        let mut x = ifloat - tmp1; 
        let mut y = jfloat - tmp2;

        if x < 0.5f64 {
          x = 0.5f64; 
        }
        if x > Nfloat + 0.5f64 {
          x = Nfloat + 0.5f64; 
        }
        let i0 = x.floor(); 
        let i1 = i0 + 1.0f64;
        if y < 0.5f64 {
          y = 0.5f64; 
        }
        if y > Nfloat + 0.5f64 {
          y = Nfloat + 0.5f64; 
        }
        let j0 = y.floor();
        let j1 = j0 + 1.0f64; 

        let s1 = x - i0; 
        let s0 = 1.0f64 - s1; 
        let t1 = y - j0; 
        let t0 = 1.0f64 - t1;

        let i0i = i0 as i32;
        let i1i = i1 as i32;
        let j0i = j0 as i32;
        let j1i = j1 as i32;

        d[IX(i, j)] = s0 * ( t0 * d0[IX(i0i, j0i)] + t1 * d0[IX(i0i, j1i)])
          + s1 * ( t0 * d0[IX(i1i, j0i)] + t1 * d0[IX(i1i, j1i)]);
      }
    }

    Fluid::set_bnd(b, d);
  }


  fn set_bnd(b: i32, x: &mut Vec<f64>) {

    for i in 1..(N - 1) {
      x[IX(i, 0)] = if b == 2 { -1f64 * x[IX(i, 1)]}else{ x[IX(i, 1)]};
      x[IX(i, N-1)] =if b == 2 { -1f64 * x[IX(i, N-2)] }else{ x[IX(i, N-2)]};
    }

    for j in 1..(N - 1) {
       x[IX(0, j)] = if b == 1 { -1f64 * x[IX(1, j)] }else{  x[IX(1, j)] };
       x[IX(N-1, j)] = if b == 1 { -1f64 * x[IX(N-2, j)] }else{  x[IX(N-2, j)]};
    }

    x[IX(0, 0)] = 0.5f64 * (x[IX(1, 0)] + x[IX(0, 1)]);
    x[IX(0, N-1)] = 0.5f64 * (x[IX(1, N-1)] + x[IX(0, N-2)]);
    x[IX(N-1, 0)] = 0.5f64 * (x[IX(N-2, 0)] + x[IX(N-1, 1)]);
    x[IX(N-1, N-1)] = 0.5f64 * (x[IX(N-2, N-1)] + x[IX(N-1, N-2)]);
  }

  // void renderD() {
  //   colorMode(HSB, 255);
  //   for (int i=0; i<N; i++) {
  //     for (int j=0; j<N; j++) {
  //       float x = i*SCALE;
  //       float y = j*SCALE;
  //       float d = this.density[IX(i, j)];
  //       noStroke();
  //       fill(d);
  //       square(x, y, SCALE);
  //     }
  //   }
  // }

  // void renderV() {
  //   for (int i=0; i<N; i++) {
  //     for (int j=0; j<N; j++) {
  //       float x = i*SCALE;
  //       float y = j*SCALE;
  //       float vx = this.Vx[IX(i, j)];
  //       float vy = this.Vy[IX(i, j)];
  //       stroke(255);
  //       if (!(abs(vx)<0.1&&abs(vy)<=0.1)) {
  //         line(x, y, x+vx*SCALE, y+vy*SCALE);
  //       }
  //     }
  //   }
  // }

  // void fadeD() {
  //   for (int i=0; i<this.density.length; i++) {
  //     float d = density[i];
  //     density[i] = constrain(d-0.1, 0, 10000);
  //   }
  // }
}


// Fluid fluid;

// float t = 0;
// int pick;

// void settings() {
//   size(N*SCALE, N*SCALE);
// }

// void setup() {
//   // Fluid(dt,density,viscosity)
//   //dt make fluid get added faster?? not sure
//   //viscosity is thickness of fluid
//   //if density is used, you do not need fade function
//   fluid = new Fluid(0.1, 0.000001, 0);
// }

// //void mouseDragged(){
// //  fluid.addDensity((mouseX/SCALE),(mouseY/SCALE),200);
// //  fluid.addColor((mouseX/SCALE),(mouseY/SCALE),pick);
// //  a: f64mtX = mouseX - pmouseX;
// //  a: f64mtY = mouseY - pmouseY;
// //  fluid.addVelocity((mouseX/SCALE),(mouseY/SCALE),amtX,amtY);
// //}

// void draw() {
//   background(0);
  
//   int cx = int(0.5*width/SCALE);
//   int cy = int(0.5*height/SCALE);
//   for (int i=-1; i<=1; i++) {
//     for (int j=-1; j<=1; j++) {
//       fluid.addDensity(cx+i, cy+j, random(6000, 10000));
//     }
//   }
//   a: f64ngle = noise(t)*TWO_PI*2;
//   PVector v = PVector.fromAngle(angle);
//   v.mult(10);
//   t += 0.01;
//   fluid.addVelocity(cx, cy, v.x, v.y);
 
  
//   fluid.step();
//   fluid.renderD();
//   //fluid.fadeD();
//   /*
//   colorMode(HSB, 255);
//   for (int i=0; i<N; i++) {
//     for (int j=0; j<N; j++) {
//       float x = i*SCALE;
//       float y = j*SCALE;
//       int index = i + N * j;
//       float d = fluid.density[index];
//       dt: f64 = fluid.density[index]-fluid.s[index];
//       noStroke();
//       fill(255*dt, d, d);
//       square(x, y, SCALE);
//     }
//   }
//   */
//   //colorMode(HSB, 255);
//   //for (int i=0; i<N; i++) {
//   //  for (int j=0; j<N; j++) {
//   //    float x = i*SCALE;
//   //    float y = j*SCALE;
//   //    int index = i + N * j;
//   //    float d = fluid.density[index];
//   //    float r = fluid.r[index];
//   //    float g = fluid.g[index];
//   //    float b = fluid.b[index];
//   //    noStroke();
//   //    fill(r, g, b);
//   //    square(x, y, SCALE);
//   //  }
//   //}
  
// }



