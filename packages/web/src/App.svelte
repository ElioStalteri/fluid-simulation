<script>
  import P5 from "p5-svelte";
  import {
    create_fluid,
    fluid_step,
    fluid_add_density,
    fluid_get_density,
    fluid_add_velocity,
    fluid_get_velocity,
  } from "vite-wasm-functions";

  let canvas_dim = 50;

  let height = 55;

  const sketch = (p5) => {
    let square_size = [];
    let density = [];
    function convertSize(x, y) {
      return [
        Math.round((x / p5.width) * canvas_dim),
        Math.round((y / p5.height) * canvas_dim),
      ];
    }

    // p5.mouseDragged = () => {
    //   console.log(...convertSize(p5.mouseX, p5.mouseY));

    // };

    // p5.mouseMoved = () => {
    //   fluid_add_density(...convertSize(p5.mouseX, p5.mouseY), 10000000);
    //   fluid_add_velocity(...convertSize(p5.mouseX, p5.mouseY), p5.random(-10,10),p5.random(-10,10));
    //   // setTimeout(() => {
    //   //   console.log(density);
    //   // });
    // };

    p5.mouseMoved = () => {
      // console.log(density);
      // console.log(fluid_get_velocity())
      fluid_add_density(...convertSize(p5.mouseX, p5.mouseY), 1000);
      fluid_add_velocity(
        ...convertSize(p5.mouseX, p5.mouseY),
        p5.mouseX - p5.pmouseX,
        p5.mouseY - p5.pmouseY
      );
      // fluid_add_velocity(
      //   ...convertSize(p5.mouseX, p5.mouseY),
      //   p5.random(-1000, 1000),
      //   p5.random(-1000, 1000)
      // );
    };

    p5.mouseClicked = () => {
      console.log(density);
      console.log(fluid_get_velocity())
      
    };

    p5.setup = () => {
      create_fluid(canvas_dim);
      p5.createCanvas(p5.windowWidth - 50, p5.windowHeight - 50);
      square_size = [p5.width / canvas_dim, p5.height / canvas_dim];
      p5.frameRate(5);
    };

    p5.draw = () => {
      // if(p5.random()>0.5)

      fluid_step();

      // @ts-ignore
      density = fluid_get_density();

      p5.background(0);
      p5.noStroke();
      for (let i = 0; i < density.length; i++) {
        const d = density[i];
        // if(d>0)console.log(d)

        const x = i % canvas_dim;
        // @ts-ignore
        const y = parseInt(i / canvas_dim);

        p5.fill(d*100);
        p5.rect(x * square_size[0], y * square_size[1], ...square_size);
        // console.log(x * square_size[0], y * square_size[1])
      }

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
