import * as THREE from 'https://unpkg.com/three@0.155.0/build/three.module.js';
import { GLTFLoader } from 'https://unpkg.com/three@0.155.0/examples/jsm/loaders/GLTFLoader.js';

const canvas   = document.getElementById('scene');
const connectBtn = document.getElementById('connect');

let renderer, scene, camera, model;

function init() {
  scene = new THREE.Scene();

  const width = window.innerWidth;
  const height = window.innerHeight;
  camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 100);
  camera.position.set(0, 0, 7);

  renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.setSize(width, height);

  // Add multiple lights for depth
  const keyLight = new THREE.DirectionalLight(0xffffff, 1.1);
  keyLight.position.set(5, 5, 10);
  scene.add(keyLight);

  const fillLight = new THREE.DirectionalLight(0xffffff, 0.6);
  fillLight.position.set(-5, -3, 5);
  scene.add(fillLight);

  const ambient = new THREE.AmbientLight(0xffffff, 0.4);
  scene.add(ambient);

  // Load the GLB model from the same folder
  const loader = new GLTFLoader();
  loader.load('blockxpand_base.glb', (gltf) => {
    model = gltf.scene;
    model.scale.set(3, 3, 3);
    // Tint the model to match your brand colours
    model.traverse((child) => {
      if (child.isMesh) {
        child.material = child.material.clone();
        child.material.color.set('#8355e2');
        child.material.emissive.set('#302070');
        child.material.emissiveIntensity = 0.5;
      }
    });
    scene.add(model);
    animate();
  });

  window.addEventListener('resize', onResize);
}

function onResize() {
  const width = window.innerWidth;
  const height = window.innerHeight;
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
  renderer.setSize(width, height);
}

function animate() {
  requestAnimationFrame(animate);
  if (model) {
    model.rotation.y += 0.015;
    model.rotation.x += 0.0075;
  }
  renderer.render(scene, camera);
}

// Connect button stub
connectBtn.addEventListener('click', async () => {
  connectBtn.disabled = true;
  const original = connectBtn.textContent;
  connectBtn.textContent = 'Connecting…';
  // TODO: integrate your wallet provider (Plug, Stoic, Internet Identity)
  await new Promise((resolve) => setTimeout(resolve, 1000));
  connectBtn.textContent = original;
  connectBtn.disabled = false;
});

init();
