# 组件使用文档

本文档详细介绍了项目中各个组件的使用方法和 API。

## 目录

- [SemiTable 表格组件](#semitable-表格组件)
- [SemiForm 表单组件](#semiform-表单组件)
- [SemiModalForm 模态框表单](#semimodalform-模态框表单)
- [AvatarDropdown 用户下拉菜单](#avatardropdown-用户下拉菜单)
- [MainLayout 主布局](#mainlayout-主布局)

## SemiTable 表格组件

基于 Semi Design Table 组件的增强版本，提供了分页、排序、搜索、操作等功能。

### 基本用法

```tsx
import { SemiTable } from '@/components';
import type { SemiTableColumn } from '@/components/SemiTable';

interface DataType {
  id: number;
  name: string;
  status: string;
  createdAt: string;
}

const columns: SemiTableColumn<DataType>[] = [
  {
    title: '名称',
    dataIndex: 'name',
    key: 'name',
  },
  {
    title: '状态',
    dataIndex: 'status',
    key: 'status',
    valueEnum: {
      active: { text: '活跃', status: 'success' },
      inactive: { text: '非活跃', status: 'danger' },
    },
  },
  {
    title: '创建时间',
    dataIndex: 'createdAt',
    key: 'createdAt',
    valueType: 'dateTime',
  },
];

<SemiTable<DataType>
  headerTitle="数据列表"
  columns={columns}
  request={async (params) => {
    const response = await fetchData(params);
    return {
      data: response.data,
      success: true,
      total: response.total,
    };
  }}
/>
```

### 带搜索功能

```tsx
<SemiTable<DataType>
  headerTitle="数据列表"
  columns={columns}
  request={fetchData}
  search={{
    labelWidth: 'auto',
    span: 8,
  }}
  searchFormRender={(form, props) => [
    <Form.Input
      key="name"
      field="name"
      label="名称"
      placeholder="请输入名称"
    />,
    <Form.Select
      key="status"
      field="status"
      label="状态"
      placeholder="请选择状态"
      optionList={[
        { label: '活跃', value: 'active' },
        { label: '非活跃', value: 'inactive' },
      ]}
    />,
  ]}
/>
```

### 可复制列

```tsx
const columns: SemiTableColumn<DataType>[] = [
  {
    title: 'URL',
    dataIndex: 'url',
    key: 'url',
    copyable: true, // 启用复制功能
  },
];
```

### API

| 属性 | 说明 | 类型 | 默认值 |
|------|------|------|--------|
| headerTitle | 表格标题 | `string` | - |
| columns | 表格列配置 | `SemiTableColumn<T>[]` | - |
| request | 数据请求函数 | `(params: any) => Promise<RequestResult<T>>` | - |
| search | 搜索配置 | `SearchConfig \| false` | `false` |
| searchFormRender | 自定义搜索表单 | `(form: FormApi, props: any) => ReactNode[]` | - |
| pagination | 分页配置 | `PaginationConfig \| false` | `{ pageSize: 10 }` |
| actionRef | 操作引用 | `MutableRefObject<ActionType>` | - |

## SemiForm 表单组件

基于 Semi Design Form 的封装组件，提供了统一的表单处理逻辑。

### 基本用法

```tsx
import { SemiForm } from '@/components';
import type { SemiFormRef } from '@/components/SemiForm/types';

const formRef = useRef<SemiFormRef>(null);

<SemiForm
  ref={formRef}
  onFinish={async (values) => {
    console.log('表单数据:', values);
    await submitData(values);
  }}
  onFinishFailed={(error) => {
    console.error('提交失败:', error);
  }}
  initialValues={{
    name: '',
    email: '',
  }}
>
  <Form.Input
    field="name"
    label="姓名"
    rules={[{ required: true, message: '请输入姓名' }]}
  />
  <Form.Input
    field="email"
    label="邮箱"
    rules={[
      { required: true, message: '请输入邮箱' },
      { type: 'email', message: '请输入有效的邮箱地址' },
    ]}
  />
  <Button htmlType="submit">提交</Button>
</SemiForm>
```

### 外部控制

```tsx
const handleSubmit = async () => {
  await formRef.current?.submit();
};

const handleReset = () => {
  formRef.current?.reset();
};

const handleSetValues = () => {
  formRef.current?.setValues({
    name: 'John Doe',
    email: 'john@example.com',
  });
};
```

### API

| 属性 | 说明 | 类型 | 默认值 |
|------|------|------|--------|
| onFinish | 表单提交成功回调 | `(values: any) => Promise<void> \| void` | - |
| onFinishFailed | 表单提交失败回调 | `(error: any) => void` | - |
| initialValues | 初始值 | `Record<string, any>` | - |
| children | 表单内容 | `ReactNode` | - |

### Ref 方法

| 方法 | 说明 | 类型 |
|------|------|------|
| submit | 提交表单 | `() => Promise<void>` |
| reset | 重置表单 | `() => void` |
| setValues | 设置表单值 | `(values: Record<string, any>) => void` |

## SemiModalForm 模态框表单

结合 Modal 和 Form 的组件，用于弹窗中的表单操作。

### 基本用法

```tsx
import { SemiModalForm } from '@/components';

const [visible, setVisible] = useState(false);

<SemiModalForm
  title="新建用户"
  visible={visible}
  onVisibleChange={setVisible}
  onFinish={async (values) => {
    await createUser(values);
    setVisible(false);
  }}
  modalProps={{
    width: 600,
    maskClosable: false,
  }}
>
  <Form.Input
    field="name"
    label="姓名"
    rules={[{ required: true, message: '请输入姓名' }]}
  />
  <Form.Input
    field="email"
    label="邮箱"
    rules={[{ required: true, message: '请输入邮箱' }]}
  />
</SemiModalForm>
```

### API

| 属性 | 说明 | 类型 | 默认值 |
|------|------|------|--------|
| title | 模态框标题 | `string` | - |
| visible | 是否显示 | `boolean` | `false` |
| onVisibleChange | 显示状态改变回调 | `(visible: boolean) => void` | - |
| onFinish | 表单提交成功回调 | `(values: any) => Promise<void> \| void` | - |
| modalProps | Modal 组件属性 | `ModalProps` | - |
| formProps | Form 组件属性 | `FormProps` | - |
| children | 表单内容 | `ReactNode` | - |

## AvatarDropdown 用户下拉菜单

用户头像和下拉菜单组件。

### 基本用法

```tsx
import { AvatarDropdown } from '@/components';

<AvatarDropdown
  currentUser={{
    name: 'John Doe',
    avatar: 'https://example.com/avatar.jpg',
    email: 'john@example.com',
  }}
  onLogout={() => {
    // 处理登出逻辑
    logout();
  }}
/>
```

### API

| 属性 | 说明 | 类型 | 默认值 |
|------|------|------|--------|
| currentUser | 当前用户信息 | `CurrentUser` | - |
| onLogout | 登出回调 | `() => void` | - |

## MainLayout 主布局

应用的主要布局组件，包含导航、侧边栏、内容区域等。

### 基本用法

```tsx
import MainLayout from '@/layouts/MainLayout';

// 在路由中使用
<Route path="/" element={<MainLayout />}>
  <Route path="dashboard" element={<Dashboard />} />
  <Route path="users" element={<Users />} />
</Route>
```

### 特性

- 响应式设计，自动适配桌面端和移动端
- 可折叠的侧边栏
- 移动端抽屉式导航
- 用户信息显示和登出功能
- 面包屑导航（可选）

### 自定义导航

导航菜单项在组件内部定义，如需修改请编辑 `src/layouts/MainLayout.tsx` 文件中的 `navItems` 配置。

```tsx
const navItems = [
  {
    itemKey: '/dashboard',
    text: '仪表盘',
    icon: <IconHome />,
  },
  {
    itemKey: '/users',
    text: '用户管理',
    icon: <IconUser />,
  },
  // 添加更多菜单项...
];
```

## 性能优化建议

### 1. 使用 React.memo

对于不经常变化的组件，使用 `React.memo` 进行优化：

```tsx
const MyComponent = React.memo(({ data }) => {
  return <div>{data.name}</div>;
});
```

### 2. 合理使用 useMemo 和 useCallback

```tsx
const ExpensiveComponent = ({ items, onItemClick }) => {
  const expensiveValue = useMemo(() => {
    return items.reduce((sum, item) => sum + item.value, 0);
  }, [items]);

  const handleClick = useCallback((id) => {
    onItemClick(id);
  }, [onItemClick]);

  return (
    <div>
      <div>Total: {expensiveValue}</div>
      {items.map(item => (
        <Item key={item.id} onClick={() => handleClick(item.id)} />
      ))}
    </div>
  );
};
```

### 3. 表格数据优化

对于大量数据的表格，考虑使用虚拟滚动或分页：

```tsx
<SemiTable
  pagination={{
    pageSize: 50, // 合理的分页大小
    showSizeChanger: true,
    showQuickJumper: true,
  }}
  scroll={{ y: 400 }} // 固定高度，启用滚动
/>
```

## 常见问题

### Q: 如何自定义表格的操作列？

A: 在 columns 配置中添加操作列：

```tsx
const columns = [
  // 其他列...
  {
    title: '操作',
    key: 'action',
    render: (text, record) => (
      <Space>
        <Button size="small" onClick={() => handleEdit(record)}>
          编辑
        </Button>
        <Button size="small" type="danger" onClick={() => handleDelete(record)}>
          删除
        </Button>
      </Space>
    ),
  },
];
```

### Q: 如何处理表单验证？

A: 使用 Semi Design 的表单验证规则：

```tsx
<Form.Input
  field="email"
  label="邮箱"
  rules={[
    { required: true, message: '请输入邮箱' },
    { type: 'email', message: '邮箱格式不正确' },
    {
      validator: (rule, value) => {
        if (value && value.length < 6) {
          return Promise.reject('邮箱长度不能少于6位');
        }
        return Promise.resolve();
      }
    }
  ]}
/>
```

### Q: 如何实现国际化？

A: 项目已配置 Semi Design 的国际化支持，如需添加更多语言：

1. 在 `src/locales/` 目录下添加语言文件
2. 在 `src/main.tsx` 中配置 LocaleProvider
3. 使用 Semi Design 的国际化 API

更多问题请查看 [Semi Design 官方文档](https://semi.design/)。
