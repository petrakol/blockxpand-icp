import * as THREE from 'https://unpkg.com/three@0.155.0/build/three.module.js';
import { GLTFLoader } from 'https://unpkg.com/three@0.155.0/examples/jsm/loaders/GLTFLoader.js';

const container = document.getElementById('scene-container');
const connectBtn = document.getElementById('connect-btn');

let renderer, scene, camera, model;

function init() {
  scene = new THREE.Scene();
  camera = new THREE.PerspectiveCamera(
    45,
    container.clientWidth / container.clientHeight,
    0.1,
    100
  );
  camera.position.set(0, 0, 4);

  renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
  renderer.setSize(container.clientWidth, container.clientHeight);
  renderer.setPixelRatio(window.devicePixelRatio);
  container.appendChild(renderer.domElement);

  const light1 = new THREE.DirectionalLight(0xffffff, 0.9);
  light1.position.set(1, 1, 2);
  scene.add(light1);

  const light2 = new THREE.AmbientLight(0xffffff, 0.6);
  scene.add(light2);

  const loader = new GLTFLoader();
  loader.load('blockXpand_base.glb', (gltf) => {
    model = gltf.scene;
    model.scale.set(1.2, 1.2, 1.2);
    scene.add(model);
    animate();
  });

  window.addEventListener('resize', onResize);
}

function onResize() {
  const w = container.clientWidth;
  const h = container.clientHeight;
  camera.aspect = w / h;
  camera.updateProjectionMatrix();
  renderer.setSize(w, h);
}

function animate() {
  requestAnimationFrame(animate);
  if (model) model.rotation.y += 0.01;
  renderer.render(scene, camera);
}

connectBtn.addEventListener('click', async () => {
  connectBtn.disabled = true;
  const original = connectBtn.textContent;
  connectBtn.textContent = 'Connecting...';
  // TODO: integrate actual wallet authentication
  await new Promise((r) => setTimeout(r, 1000));
  connectBtn.textContent = original;
  connectBtn.disabled = false;
});

init();
