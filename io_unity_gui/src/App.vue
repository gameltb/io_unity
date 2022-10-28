<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { ElContainer, ElAside, ElDivider, ElHeader, ElMain, ElTable, ElTableColumn, ElFooter, ElInput } from "element-plus";
import { computed, ref, Suspense } from "vue";
import Tree from "./components/Tree.vue";

const changehandle = async (cab: string) => {
  const objects = await invoke("list_fs_cab", { fsPath: cab })
  tableData.value = objects
};

const tableData = ref([])

const search = ref('')
const filterTableData = computed(() =>
  tableData.value.filter(
    (data) =>
      !search.value ||
      data.name.toLowerCase().includes(search.value.toLowerCase())
  )
)
</script>

<template>
  <div class="common-layout" style="height: 100%;">
    <el-container style="height: 100%;">
      <el-aside width="30%">
        <Suspense>
          <Tree @selected-cab="changehandle" />
        </Suspense>
      </el-aside>
      <el-divider direction="vertical" style="height: 100%;" />
      <el-container>
        <el-header>
          <el-input v-model="search" placeholder="search name" />
        </el-header>
        <el-main>
          <el-table :data="filterTableData" stripe height="100%" style="width: 100%">
            <el-table-column prop="path_id" label="Path ID" />
            <el-table-column prop="tp" label="Type" />
            <el-table-column prop="name" label="Name" />
          </el-table>
        </el-main>
        <!-- <el-footer>Footer</el-footer> -->
      </el-container>
    </el-container>
  </div>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
