final int Nx = 100;
final int Ny = 100;
final int iter = 1;
final int SCALE = 4;

int IX(int x, int y) {
  x = constrain(x, 0, Nx-1);
  y = constrain(y, 0, Ny-1);
  return x + (y * Nx);
}

class Fluid {
  int size;
  float dt; //time step
  float diff; //diffusion amount
  float visc; //thickness of fluid

  float[] s; //previous density
  float[] density;

  float[] Vx;
  float[] Vy;

  float[] Vx0;
  float[] Vy0;

  float[] r;
  float[] g;
  float[] b;

  Fluid(float dt, float diffusion, float viscosity) {
    this.dt = dt;
    this.diff = diffusion;
    this.visc = viscosity;

    this.s = new float[Nx*Ny];
    this.density = new float[Nx*Ny];

    this.Vx = new float[Nx*Ny];
    this.Vy = new float[Nx*Ny];

    this.Vx0 = new float[Nx*Ny];
    this.Vy0 = new float[Nx*Ny];
  }

  void step() {
    float visc     = this.visc;
    float diff     = this.diff;
    float dt       = this.dt;
    float[] Vx      = this.Vx;
    float[] Vy      = this.Vy;
    float[] Vx0     = this.Vx0;
    float[] Vy0     = this.Vy0;
    float[] s       = this.s;
    float[] density = this.density;

    diffuse(1, Vx0, Vx, visc, dt);
    diffuse(2, Vy0, Vy, visc, dt);
    
    project(Vx0, Vy0, Vx, Vy);

    advect(1, Vx, Vx0, Vx0, Vy0, dt);
    advect(2, Vy, Vy0, Vx0, Vy0, dt);

    project(Vx, Vy, Vx0, Vy0);
    
    diffuse(0, s, density, diff, dt);
    advect(0, density, s, Vx, Vy, dt);
 
  }

  void addDensity(int x, int y, float amount) {
    int index = IX(x, y);
    this.density[index] += amount;
  }


  void addVelocity(int x, int y, float amountX, float amountY) {
    int index = IX(x, y);
    this.Vx[index] += amountX;
    this.Vy[index] += amountY;
  }

  void renderD() {
    colorMode(HSB, 255);
    for (int i=0; i<Nx; i++) {
      for (int j=0; j<Ny; j++) {
        float x = i*SCALE;
        float y = j*SCALE;
        float d = this.density[IX(i, j)];
        noStroke();
        fill(d);
        square(x, y, SCALE);
      }
    }
  }

  void renderV() {
    for (int i=0; i<Nx; i++) {
      for (int j=0; j<Ny; j++) {
        float x = i*SCALE;
        float y = j*SCALE;
        float vx = this.Vx[IX(i, j)];
        float vy = this.Vy[IX(i, j)];
        stroke(255);
        if (!(abs(vx)<0.1&&abs(vy)<=0.1)) {
          line(x, y, x+vx*SCALE, y+vy*SCALE);
        }
      }
    }
  }

  void fadeD() {
    for (int i=0; i<this.density.length; i++) {
      float d = density[i];
      density[i] = constrain(d-0.1, 0, 10000);
    }
  }
}

void diffuse(int b, float[] x, float[] x0, float diff, float dt) {
  float a = dt * diff * (Nx - 2) * (Ny - 2);
  lin_solve(b, x, x0, a, 1 + 6 * a);
}

void lin_solve(int b, float[] x, float[] x0, float a, float c) {
  float cRecip = 1.0 / c;
  for (int k = 0; k < iter; k++) {
    for (int j = 1; j < Ny - 1; j++) {
      for (int i = 1; i < Nx - 1; i++) {
        x[IX(i, j)] =
          (x0[IX(i, j)]
          + a*(    x[IX(i+1, j)]
          +x[IX(i-1, j)]
          +x[IX(i, j+1)]
          +x[IX(i, j-1)]
          )) * cRecip;
      }
    }
    //set_bnd(b, x);
  }
}

void project(float[] velocX, float[] velocY, float[] p, float[] div) {
  for (int j = 1; j < Ny - 1; j++) {
    for (int i = 1; i < Nx - 1; i++) {
      div[IX(i, j)] = -0.5f*(
        (velocX[IX(i+1, j)]-velocX[IX(i-1, j)])/Nx + 
        (velocY[IX(i, j+1)]-velocY[IX(i, j-1)])/Ny
        );
      p[IX(i, j)] = 0;
    }
  }
  set_bnd(0, div); 
  set_bnd(0, p);
  lin_solve(0, p, div, 1, 6);

  for (int j = 1; j < Ny - 1; j++) {
    for (int i = 1; i < Nx - 1; i++) {
      velocX[IX(i, j)] -= 0.5f * (  p[IX(i+1, j)]
        -p[IX(i-1, j)]) * Nx;
      velocY[IX(i, j)] -= 0.5f * (  p[IX(i, j+1)]
        -p[IX(i, j-1)]) * Ny;
    }
  }
  set_bnd(1, velocX);
  set_bnd(2, velocY);
}


void advect(int b, float[] d, float[] d0, float[] velocX, float[] velocY, float dt)
{
  float i0, i1, j0, j1;

  float dtx = dt * (Nx - 2);
  float dty = dt * (Ny - 2);

  float s0, s1, t0, t1;
  float tmp1, tmp2, x, y;

  float Nxfloat = Nx;
  float Nyfloat = Ny;
  float ifloat, jfloat;
  int i, j;

  for (j = 1, jfloat = 1; j < Ny - 1; j++, jfloat++) { 
    for (i = 1, ifloat = 1; i < Nx - 1; i++, ifloat++) {
      tmp1 = dtx * velocX[IX(i, j)];
      tmp2 = dty * velocY[IX(i, j)];
      x    = ifloat - tmp1; 
      y    = jfloat - tmp2;

      if (x < 0.5f) x = 0.5f; 
      if (x > Nxfloat + 0.5f) x = Nxfloat + 0.5f; 
      i0 = floor(x); 
      i1 = i0 + 1.0f;
      if (y < 0.5f) y = 0.5f; 
      if (y > Nyfloat + 0.5f) y = Nyfloat + 0.5f; 
      j0 = floor(y);
      j1 = j0 + 1.0f; 

      s1 = x - i0; 
      s0 = 1.0f - s1; 
      t1 = y - j0; 
      t0 = 1.0f - t1;

      int i0i = int(i0);
      int i1i = int(i1);
      int j0i = int(j0);
      int j1i = int(j1);

      d[IX(i, j)] = s0 * ( t0 * d0[IX(i0i, j0i)] + t1 * d0[IX(i0i, j1i)])
        + s1 * ( t0 * d0[IX(i1i, j0i)] + t1 * d0[IX(i1i, j1i)]);
    }
  }

  set_bnd(b, d);
}


void set_bnd(int b, float[] x) {

  for (int i = 1; i < Nx - 1; i++) {
    x[IX(i, 0)] = b == 2 ? -x[IX(i, 1)] : x[IX(i, 1)];
    x[IX(i, Ny-1)] = b == 2 ? -x[IX(i, Ny-2)] : x[IX(i, Ny-2)];
  }

  for (int j = 1; j < Ny - 1; j++) {
    x[IX(0, j)] = b == 1 ? -x[IX(1, j)] : x[IX(1, j)];
    x[IX(Nx-1, j)] = b == 1 ? -x[IX(Nx-2, j)] : x[IX(Nx-2, j)];
  }

  x[IX(0, 0)] = 0.5f * (x[IX(1, 0)] + x[IX(0, 1)]);
  x[IX(0, Ny-1)] = 0.5f * (x[IX(1, Ny-1)] + x[IX(0, Ny-2)]);
  x[IX(Nx-1, 0)] = 0.5f * (x[IX(Nx-2, 0)] + x[IX(Nx-1, 1)]);
  x[IX(Nx-1, Ny-1)] = 0.5f * (x[IX(Nx-2, Ny-1)] + x[IX(Nx-1, Ny-2)]);
}



Fluid fluid;

float t = 0;
int pick;

void settings() {
  size(Nx*SCALE, Ny*SCALE);
}

void setup() {
  // Fluid(dt,density,viscosity)
  //dt make fluid get added faster?? not sure
  //viscosity is thickness of fluid
  //if density is used, you do not need fade function
  fluid = new Fluid(0.1, 0.000001, 0);
}

//void mouseDragged(){
//  fluid.addDensity((mouseX/SCALE),(mouseY/SCALE),200);
//  fluid.addColor((mouseX/SCALE),(mouseY/SCALE),pick);
//  float amtX = mouseX - pmouseX;
//  float amtY = mouseY - pmouseY;
//  fluid.addVelocity((mouseX/SCALE),(mouseY/SCALE),amtX,amtY);
//}

void draw() {
  background(0);
  
  int cx = int(0.5*width/SCALE);
  int cy = int(0.5*height/SCALE);
  for (int i=-1; i<=1; i++) {
    for (int j=-1; j<=1; j++) {
      fluid.addDensity(cx+i, cy+j, random(6000, 10000));
    }
  }
  float angle = noise(t)*TWO_PI*2;
  PVector v = PVector.fromAngle(angle);
  v.mult(10);
  t += 0.01;
  fluid.addVelocity(cx, cy, v.x, v.y);
 
  
  fluid.step();
  fluid.renderD();
  //fluid.fadeD();
  /*
  colorMode(HSB, 255);
  for (int i=0; i<Nx; i++) {
    for (int j=0; j<Ny; j++) {
      float x = i*SCALE;
      float y = j*SCALE;
      int index = i + Nx * j;
      float d = fluid.density[index];
      float dt = fluid.density[index]-fluid.s[index];
      noStroke();
      fill(255*dt, d, d);
      square(x, y, SCALE);
    }
  }
  */
  //colorMode(HSB, 255);
  //for (int i=0; i<Nx; i++) {
  //  for (int j=0; j<Ny; j++) {
  //    float x = i*SCALE;
  //    float y = j*SCALE;
  //    int index = i + Nx * j;
  //    float d = fluid.density[index];
  //    float r = fluid.r[index];
  //    float g = fluid.g[index];
  //    float b = fluid.b[index];
  //    noStroke();
  //    fill(r, g, b);
  //    square(x, y, SCALE);
  //  }
  //}
  
}



