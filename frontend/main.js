import * as THREE from 'https://unpkg.com/three@0.155.0/build/three.module.js';
import { GLTFLoader } from 'https://unpkg.com/three@0.155.0/examples/jsm/loaders/GLTFLoader.js';

console.log('🔔 main.js loaded');

const canvas     = document.getElementById('scene');   // <canvas id="scene">
const connectBtn = document.getElementById('connect'); // <button id="connect">
let   renderer, scene, camera, model;

/**
 * Initialise Three.js renderer, camera, lights
 */
function init() {
  scene = new THREE.Scene();

  const width  = window.innerWidth;
  const height = window.innerHeight;

  camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 100);
  camera.position.set(0, 0, 7);

  renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.setSize(width, height);

  /* Key / fill / ambient lights */
  const keyLight   = new THREE.DirectionalLight(0xffffff, 1.1);
  keyLight.position.set(5, 5, 10);
  scene.add(keyLight);

  const fillLight  = new THREE.DirectionalLight(0xffffff, 0.6);
  fillLight.position.set(-5, -3, 5);
  scene.add(fillLight);

  scene.add(new THREE.AmbientLight(0xffffff, 0.4));

  loadModel();
  window.addEventListener('resize', onResize);
}

/**
 * Load GLB (Draco or non‑Draco)
 */
async function loadModel() {
  /* Fetch once to verify availability & MIME */
  try {
    const r = await fetch('blockxpand_base.glb');
    console.log('ℹ️ fetch', r.status, r.headers.get('content-type'));
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
  } catch (e) {
    console.error('❌ Cannot fetch blockxpand_base.glb – check path / headers', e);
    return;
  }

  const loader = new GLTFLoader();

  /* Attach Draco loader (harmless if model is not Draco‑compressed) */
  const { DRACOLoader } = await import(
    'https://unpkg.com/three@0.155.0/examples/jsm/loaders/DRACOLoader.js'
  );
  const draco = new DRACOLoader();
  draco.setDecoderPath('https://www.gstatic.com/draco/v1/decoders/');
  loader.setDRACOLoader(draco);

  loader.load(
    'blockxpand_base.glb',
    (gltf) => {
      console.log('✅ GLB parsed');
      model = gltf.scene;

      /* If artist exported nested node, descend to first child with geometry */
      if (!model.children.length && model.scene) model = model.scene;
      if (model.children.length === 1 && !model.children[0].isMesh) {
        model = model.children[0];
      }

      /* Brand‑purple material (MeshStandard) */
      model.traverse((c) => {
        if (c.isMesh) {
          c.material = new THREE.MeshStandardMaterial({
            color: 0x8355e2,
            metalness: 0.4,
            roughness: 0.4,
          });
        }
      });

      /* Scale, centre, and frame */
      const box    = new THREE.Box3().setFromObject(model);
      const size   = box.getSize(new THREE.Vector3()).length() || 1;
      const center = box.getCenter(new THREE.Vector3());
      model.position.sub(center);          // centre the geometry
      const fovRad = (camera.fov * Math.PI) / 180;
      const dist   = (size * 0.6) / Math.tan(fovRad / 2);
      camera.position.set(0, 0, dist);
      camera.lookAt(0, 0, 0);

      scene.add(model);
      animate();
    },
    undefined,
    (err) => console.error('❌ GLB load error', err)
  );
}

/**
 * Responsive resize
 */
function onResize() {
  const w = window.innerWidth;
  const h = window.innerHeight;
  camera.aspect = w / h;
  camera.updateProjectionMatrix();
  renderer.setSize(w, h);
}

/**
 * Animation loop
 */
function animate() {
  requestAnimationFrame(animate);
  if (model) {
    model.rotation.y += 0.015;
    model.rotation.x += 0.0075;
  }
  renderer.render(scene, camera);
}

/* Stub “Connect Wallet” button */
connectBtn.addEventListener('click', async () => {
  connectBtn.disabled = true;
  const original = connectBtn.textContent;
  connectBtn.textContent = 'Connecting…';
  await new Promise((r) => setTimeout(r, 1200)); // simulate delay
  connectBtn.textContent = original;
  connectBtn.disabled = false;
});

init();