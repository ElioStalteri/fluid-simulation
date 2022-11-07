<script>
  import P5 from "p5-svelte";
  import {
    create_fluid,
    fluid_step,
    fluid_add_density,
    fluid_get_density,
    fluid_add_velocity,
    get_size,
    // fluid_get_velocity,
  } from "vite-wasm-functions";

  let canvas_dim = 150;

  const sketch = (p5) => {
    let square_size = [];
    let density = [];
    function convertSize(x, y) {
      return [
        Math.round((x / p5.width) * canvas_dim),
        Math.round((y / p5.height) * canvas_dim),
      ];
    }

    p5.mouseClicked = ()=>{
      fluid_add_density(...convertSize(p5.mouseX, p5.mouseY), 1000);
    }

    p5.mouseMoved = () => {
      fluid_add_velocity(
        ...convertSize(p5.mouseX, p5.mouseY),
        p5.mouseX - p5.pmouseX,
        p5.mouseY - p5.pmouseY
      );
    };

    p5.setup = () => {
      create_fluid();
      canvas_dim = get_size();
      p5.createCanvas(p5.windowWidth - 50, p5.windowHeight - 50);
      square_size = [p5.width / canvas_dim, p5.height / canvas_dim];
    };

    const addSmoke = (percx, percy, t, modifyDirection) => {
      const [cx, cy] = convertSize(percx * p5.width, percy * p5.height);
      for (let i = -1; i <= 1; i++) {
        for (let j = -1; j <= 1; j++) {
          fluid_add_density(cx + i, cy + j, 255);
        }
      }
      const angle = p5.noise(t) * p5.TWO_PI * 2;
      const v = p5.Vector.fromAngle(angle - modifyDirection);
      v.mult(p5.random(50, 500));
      fluid_add_velocity(cx, cy, v.x, v.y);
    };

    // let t = 0;
    p5.draw = () => {
      // if(p5.random()>0.5)
      // addSmoke(0.5, 0.5, t, p5.random(0, p5.TWO_PI));
      // addSmoke(0.1, 0.1, t, 0);
      // addSmoke(0.1, 0.9, t, p5.PI / 2);
      // addSmoke(0.9, 0.1, t, p5.PI);
      // addSmoke(0.9, 0.9, t, (p5.PI * 3) / 4);
      // t += 0.05;
      // console.time("step");
      fluid_step();
      // console.timeEnd("step");

      // console.time("getArray");
      // @ts-ignore
      density = fluid_get_density();
      // console.timeEnd("getArray");

      p5.background(0);
      p5.noStroke();
      // console.time("printArray");
      for (let i = 0; i < density.length; i++) {
        const d = density[i];
        // if(d>0)console.log(d)

        const x = i % canvas_dim;
        // @ts-ignore
        const y = parseInt(i / canvas_dim);

        p5.fill(d);
        p5.rect(x * square_size[0], y * square_size[1], ...square_size);
        // console.log(x * square_size[0], y * square_size[1])
      }
      // console.timeEnd("printArray");

      // p5.noLoop();
    };
  };
</script>

<!-- <label>
  Width
  <input type="range" bind:value={width} min="100" max="1000" step="0.01" />
  {width}
</label>

<label>
  Height
  <input type="range" bind:value={height} min="100" max="1000" step="0.01" />
  {height}
</label> -->

<P5 {sketch} />
