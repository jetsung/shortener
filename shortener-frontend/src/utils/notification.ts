// 自定义通知组件
// 替代 Semi Design Toast 组件

type NotificationType = 'success' | 'error' | 'warning' | 'info';

interface NotificationConfig {
  message: string;
  duration?: number;
  type: NotificationType;
}

class NotificationManager {
  private container: HTMLElement | null = null;
  private notifications: Map<string, HTMLElement> = new Map();
  private lastNotificationId: string | null = null;

  private getContainer(): HTMLElement {
    if (!this.container) {
      this.container = document.createElement('div');
      this.container.style.cssText = `
        position: fixed;
        top: 20px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 9999;
        pointer-events: none;
        display: flex;
        flex-direction: column;
        align-items: center;
      `;
      document.body.appendChild(this.container);
    }
    return this.container;
  }

  private createNotification(config: NotificationConfig): HTMLElement {
    const { message, type, duration = 3000 } = config;

    const notification = document.createElement('div');
    const id = Math.random().toString(36).substr(2, 9);

    // 设置样式
    notification.style.cssText = `
      background: white;
      border-radius: 6px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
      padding: 12px 16px;
      margin-bottom: 8px;
      max-width: 300px;
      min-width: 200px;
      word-wrap: break-word;
      pointer-events: auto;
      cursor: pointer;
      transition: all 0.3s ease;
      transform: translateY(-20px);
      opacity: 0;
      border-left: 4px solid ${this.getTypeColor(type)};
      display: flex;
      align-items: center;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
      font-size: 14px;
      line-height: 1.5;
    `;

    // 添加图标和消息
    const icon = this.getTypeIcon(type);
    notification.innerHTML = `
      <span style="margin-right: 8px; font-size: 16px;">${icon}</span>
      <span>${message}</span>
    `;

    // 点击关闭
    notification.addEventListener('click', () => {
      this.removeNotification(id);
    });

    // 添加到容器
    const container = this.getContainer();
    container.appendChild(notification);
    this.notifications.set(id, notification);

    // 记录最后一个通知的 ID
    this.lastNotificationId = id;

    // 动画进入
    requestAnimationFrame(() => {
      notification.style.transform = 'translateY(0)';
      notification.style.opacity = '1';
    });

    // 自动移除
    setTimeout(() => {
      this.removeNotification(id);
    }, duration);

    return notification;
  }

  private updateNotification(id: string, config: NotificationConfig): void {
    const notification = this.notifications.get(id);
    if (!notification) return;

    const { message, type } = config;

    // 更新样式
    notification.style.borderLeftColor = this.getTypeColor(type);

    // 更新内容
    const icon = this.getTypeIcon(type);
    notification.innerHTML = `
      <span style="margin-right: 8px; font-size: 16px;">${icon}</span>
      <span>${message}</span>
    `;

    // 重置自动移除定时器
    setTimeout(() => {
      this.removeNotification(id);
    }, 3000);
  }

  private removeNotification(id: string): void {
    const notification = this.notifications.get(id);
    if (notification) {
      notification.style.transform = 'translateY(-20px)';
      notification.style.opacity = '0';

      setTimeout(() => {
        if (notification.parentNode) {
          notification.parentNode.removeChild(notification);
        }
        this.notifications.delete(id);

        // 如果没有通知了，移除容器
        if (this.notifications.size === 0 && this.container) {
          document.body.removeChild(this.container);
          this.container = null;
        }
      }, 300);
    }
  }

  private getTypeColor(type: NotificationType): string {
    const colors = {
      success: '#52c41a',
      error: '#ff4d4f',
      warning: '#faad14',
      info: '#1890ff',
    };
    return colors[type];
  }

  private getTypeIcon(type: NotificationType): string {
    const icons = {
      success: '✓',
      error: '✕',
      warning: '⚠',
      info: 'ℹ',
    };
    return icons[type];
  }

  public success(message: string, duration?: number): void {
    this.createNotification({ message, type: 'success', duration });
  }

  public error(message: string, duration?: number): void {
    this.createNotification({ message, type: 'error', duration });
  }

  public warning(message: string, duration?: number): void {
    this.createNotification({ message, type: 'warning', duration });
  }

  public info(message: string, duration?: number): void {
    this.createNotification({ message, type: 'info', duration });
  }

  public updateLast(message: string, type: NotificationType): void {
    if (this.lastNotificationId) {
      this.updateNotification(this.lastNotificationId, { message, type });
    } else {
      this.createNotification({ message, type });
    }
  }
}

// 创建单例实例
const notificationManager = new NotificationManager();

// 导出与 Semi Design Toast 兼容的 API
export const Toast = {
  success: (message: string, duration?: number) => notificationManager.success(message, duration),
  error: (message: string, duration?: number) => notificationManager.error(message, duration),
  warning: (message: string, duration?: number) => notificationManager.warning(message, duration),
  info: (message: string, duration?: number) => notificationManager.info(message, duration),
  update: (message: string, type: NotificationType) =>
    notificationManager.updateLast(message, type),
};

export default Toast;
