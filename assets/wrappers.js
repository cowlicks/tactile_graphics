import * as THREE from '/assets/three.module.js';
import { OrbitControls } from '/assets/OrbitControls.js';
import { STLLoader } from '/assets/STLLoader.js';

import {
  Canvg,
  presets
} from 'https://cdn.skypack.dev/canvg@^4.0.0';

const preset = presets.offscreen()

const default_svg_to_png_height = 600;
const default_svg_to_png_width = 600;

async function svg_string_to_bitmap_data_url(svg_string) {
  const height = default_svg_to_png_height;
  const width = default_svg_to_png_width;
  const canvas = new OffscreenCanvas(width, height)
  const ctx = canvas.getContext('2d')
  const v = await Canvg.from(ctx, svg_string, preset)

  // Render only first frame, ignoring animations and mouse.
  await v.render()

  const blob = await canvas.convertToBlob()
  const pngUrl = URL.createObjectURL(blob)

  return pngUrl
}

async function testCanvg() {
  async function dlToSting() {
    return new Promise(resolve => {
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
          resolve(xhttp.responseText);
        }
    };
    xhttp.open("GET", "/example.svg", true);
    xhttp.send();
    });
  }

  let svg = await dlToSting();
  console.log('got svg string', svg.slice(0, 20));
  let pngUrl = await svg_string_to_bitmap_data_url(
    svg,
  );

  const img = document.createElement('img');
  img.src = pngUrl;
  console.log(pngUrl);
  document.getElementsByTagName('body')[0].appendChild(img);

}

//testCanvg();

export function canvas_from_image(durl) {
  console.log('from javascript');
  return new Promise((resolve) => {
    const newimg = new Image();
    newimg.style.display = "none";
    newimg.onload = () => {
      const canvas = document.createElement('canvas');
      canvas.width = newimg.width;
      canvas.height = newimg.height;
      console.log(newimg.width);
      console.log(newimg.width);

      console.log(canvas.width);
      console.log(canvas.width);

      const ctx = canvas.getContext("2d");
      ctx.drawImage(newimg, 0, 0);
      console.log('sending canvas over');
      resolve(canvas);
    }
    newimg.src = durl;
  });
}

export function insert_canvas(durl, canvas_id) {
  const newimg = new Image();
  newimg.style.display = "none";
  newimg.onload = () => {
    const canvas = document.getElementById(canvas_id);
    canvas.width = newimg.width;
    canvas.height = newimg.height;

    const ctx = canvas.getContext("2d");
    ctx.drawImage(newimg, 0, 0);
  }
  newimg.src = durl;
}

// model should be a File object
export function STLViewer(model, elementID) {
  model = URL.createObjectURL(model);
  document.getElementById('download-button').href = model;

  var elem = document.getElementById(elementID)

  var camera = new THREE.PerspectiveCamera(70, 
    elem.clientWidth/elem.clientHeight, 1, 1000);

  var renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
  renderer.setSize(elem.clientWidth, elem.clientHeight);
  elem.innerHTML = '';
  elem.appendChild(renderer.domElement);

  var controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.rotateSpeed = 0.10;
  controls.dampingFactor = 0.1;
  controls.enableZoom = true;
  controls.autoRotate = true;
  controls.autoRotateSpeed = .75;

  var scene = new THREE.Scene();
  scene.add(new THREE.HemisphereLight(0xffffff, 1.5));

  (new STLLoader()).load(model, function (geometry) {

    geometry.computeVertexNormals();

    var material = new THREE.MeshToonMaterial({ 
        color: 0xff5533, 
        side: THREE.DoubleSide,
    });

    var mesh = new THREE.Mesh(geometry, material);
        scene.add(mesh);

    var middle = new THREE.Vector3();
    geometry.computeBoundingBox();
    geometry.boundingBox.getCenter(middle);
    mesh.geometry.applyMatrix4(new THREE.Matrix4().makeTranslation(
                                    -middle.x, -middle.y, -middle.z ) );

    var largestDimension = Math.max(geometry.boundingBox.max.x,
                            geometry.boundingBox.max.y,
                            geometry.boundingBox.max.z)
    camera.position.z = largestDimension * 1.5;

    var animate = function () {
      requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    };

        animate();
    });
}
