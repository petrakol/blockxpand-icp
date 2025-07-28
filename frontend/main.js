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
  loader.load(
    'blockxpand_base.glb',
    (gltf) => {
      console.log('✅ model loaded');

      model = gltf.scene;
      model.scale.set(3, 3, 3);

      // Make every mesh HOT-PINK wireframe so it can't hide
      model.traverse((c) => {
        if (c.isMesh) {
          c.material = new THREE.MeshBasicMaterial({
            color: 0xff00ff,
            wireframe: true,
          });
        }
      });

      // Force grey background so we can see opaque black covers
      scene.background = new THREE.Color('#202020');

      // Center & frame
      const box = new THREE.Box3().setFromObject(model);
      const size = box.getSize(new THREE.Vector3()).length() || 1;
      const center = box.getCenter(new THREE.Vector3());
      model.position.sub(center);

      const fovRad = camera.fov * (Math.PI / 180);
      const dist = (size * 0.6) / Math.tan(fovRad / 2);
      camera.position.set(0, 0, dist);
      camera.lookAt(0, 0, 0);

      scene.add(model);
      animate();
    },
    undefined,
    (err) => console.error('❌ GLB load error:', err)
  );

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
