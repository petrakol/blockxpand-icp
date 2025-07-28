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

  /* ----------------------------------------------------------------------------
     1) NETWORK PROBE – fetch the GLB “by hand” to confirm 200 + correct MIME
  --------------------------------------------------------------------------- */
  fetch('blockxpand_base.glb')
    .then((r) => {
      console.log('ℹ️ fetch status', r.status, r.headers.get('content-type'));
      return r.arrayBuffer();
    })
    .catch((err) => console.error('❌ fetch failed', err));

  /* ----------------------------------------------------------------------------
     2) GLB LOAD (with DRACOLoader support in case the file is Draco‑compressed)
  --------------------------------------------------------------------------- */
  const loader = new GLTFLoader();
  import('https://unpkg.com/three@0.155.0/examples/jsm/loaders/DRACOLoader.js').then(
    (mod) => {
      const draco = new mod.DRACOLoader();
      draco.setDecoderPath('https://www.gstatic.com/draco/v1/decoders/');
      loader.setDRACOLoader(draco);

      loader.load(
        'blockxpand_base.glb',
        (gltf) => {
          console.log('✅ GLB parsed');
          model = gltf.scene;
          model.scale.set(3, 3, 3);

          /* ***** DEBUG MATERIAL – hot pink wireframe ***** */
          model.traverse((c) => {
            if (c.isMesh) {
              c.material = new THREE.MeshBasicMaterial({
                color: 0xff00ff,
                wireframe: true,
              });
            }
          });

          /* 2a) GEOMETRY PROBE – print bounding‑box length */
          const box = new THREE.Box3().setFromObject(model);
          let size = box.getSize(new THREE.Vector3()).length();
          let center;
          console.log('📐 GLB size (bbox length):', size);

          if (size < 0.001) {
            console.warn(
              '⚠️ size ~0 – maybe the geometry is nested; using first child'
            );
            if (model.children[0]) {
              model = model.children[0];
              box.setFromObject(model);
              size = box.getSize(new THREE.Vector3()).length();
              center = box.getCenter(new THREE.Vector3());
            }
          }

          /* Center / frame */
          center = center || box.getCenter(new THREE.Vector3());
          model.position.sub(center);
          const dist = (size || 5) / Math.tan((camera.fov * Math.PI) / 180 / 2);
          camera.position.set(0, 0, dist);

          scene.add(model);
          animate();
        },
        undefined,
        (err) => console.error('❌ GLB load error:', err)
      );
    }
  );

  /* ----------------------------------------------------------------------------
     3) FALLBACK CUBE – always visible if renderer works
  --------------------------------------------------------------------------- */
  const debugCube = new THREE.Mesh(
    new THREE.BoxGeometry(1, 1, 1),
    new THREE.MeshBasicMaterial({ color: 0x00ff00, wireframe: true })
  );
  debugCube.position.set(-3, 0, 0);
  scene.add(debugCube);

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
