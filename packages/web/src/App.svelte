<script>
  import P5 from "p5-svelte";
  import {
    create_fluid,
    fluid_step,
    fluid_add_density,
  } from "vite-wasm-functions";

  let square_side = 255;
  let height = 55;

  const sketch = (p5) => {
    function convertSize(x, y) {
      return [
        Math.round((x / p5.width) * square_side),
        Math.round((y / p5.height) * square_side),
      ];
    }

    p5.mouseDragged = () => {
      console.log(...convertSize(p5.mouseX, p5.mouseY));
      // fluid_add_density
    };

    p5.setup = () => {
      create_fluid(square_side);
      p5.createCanvas(p5.windowWidth - 50, p5.windowHeight - 50);
    };

    p5.draw = () => {
      fluid_step();
      p5.background(0);
      p5.noLoop();
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
