/* ------------------------------------------------------------------
   BlockXpand hero – minimal, working, CDN‑based Three.js loader
------------------------------------------------------------------ */
import * as THREE      from 'https://unpkg.com/three@0.155.0/build/three.module.js';
import { GLTFLoader }  from 'https://unpkg.com/three@0.155.0/examples/jsm/loaders/GLTFLoader.js';

const canvas = document.getElementById('scene');
const btn    = document.getElementById('connect');

let renderer, scene, camera, model;

function init () {
  scene   = new THREE.Scene();
  camera  = new THREE.PerspectiveCamera(45, innerWidth/innerHeight, 0.1, 100);
  camera.position.set(0,0,7);

  renderer = new THREE.WebGLRenderer({canvas, alpha:true, antialias:true});
  renderer.setPixelRatio(devicePixelRatio);
  renderer.setSize(innerWidth, innerHeight);

  scene.add(new THREE.AmbientLight(0xffffff, .7));
  const key = new THREE.DirectionalLight(0xffffff, 1.2);
  key.position.set(5,6,8); scene.add(key);

  loadGLB();
  addListeners();
}

async function loadGLB () {
  const loader = new GLTFLoader();
  loader.load('blockxpand_base.glb', (gltf)=>{
      model = gltf.scene;
      model.traverse(m=>{
        if (m.isMesh) m.material = new THREE.MeshStandardMaterial({color:0x8355e2,metalness:.4,roughness:.4});
      });

      // centre & frame
      const box    = new THREE.Box3().setFromObject(model);
      const size   = box.getSize(new THREE.Vector3()).length() || 1;
      const center = box.getCenter(new THREE.Vector3());
      model.position.sub(center);
      camera.position.z = (size*0.7)/Math.tan(Math.PI*camera.fov/360);

      scene.add(model);
      animate();
  },undefined,e=>console.error('GLB load error',e));
}

function animate () {
  requestAnimationFrame(animate);
  if (model){ model.rotation.y += .012; model.rotation.x += .006; }
  renderer.render(scene,camera);
}

function addListeners () {
  addEventListener('resize',()=>{
    camera.aspect = innerWidth/innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(innerWidth,innerHeight);
  });

  btn.addEventListener('click',async()=>{
    const txt = btn.textContent;
    btn.disabled = true; btn.textContent='Connecting…';
    await new Promise(r=>setTimeout(r,1200));
    btn.textContent = txt; btn.disabled = false;
  });
}

init();
