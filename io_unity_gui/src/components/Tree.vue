<template>
  <el-tree ref="treeRef" :props="props" :load="loadNode" :key="refresh" @node-click="changehandle" lazy show-checkbox />
</template>

<script lang="ts" setup>
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri';
import { ElTree, ElButton } from 'element-plus';
import type Node from 'element-plus/es/components/tree/src/model/node'
import { ref } from 'vue';

const treeRef = ref<InstanceType<typeof ElTree>>()

interface Tree {
  name: string
}

const props = {
  label: 'name',
  children: 'zones',
  isLeaf: 'leaf',
}

const loadNode = async (node: Node, resolve: (data: Tree[]) => void) => {
  if (node.level === 0) {
    const data: string[] = await invoke("list_fs")
    const treedata: Tree[] = data.map(s => { return { name: s }; })
    return resolve(treedata)
  } else if (node.level === 1) {
    const data: string[] = await invoke("list_fs_path", { fsPath: node.data.name })
    const treedata: Tree[] = data.map(s => { return { name: s }; })
    return resolve(treedata)
  }
  return resolve([])
}

const refresh = ref("");

const menu_open_event_listen = await listen('menu-open-event', (event) => {
  // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
  // event.payload is the payload object
  console.log(event)
  Promise.all([invoke("open_fs", { fsPath: event.payload })])
  refresh.value = Date.now()
  emit('selectedCab', event.payload);
});

listen('tauri://file-drop', event => {
  console.log(event)
  Promise.all([invoke("open_fs", { fsPath: event.payload[0] })])
  refresh.value = Date.now()
  emit('selectedCab', event.payload[0]);
})

const emit = defineEmits(['selectedCab']);

const changehandle = (tree: Tree, node: Node) => {
  if (node.level === 1) {
    emit('selectedCab', tree.name);
  }
};
</script>