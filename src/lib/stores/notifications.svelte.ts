export type NotificationType = 'success' | 'error' | 'info' | 'warning';

export interface Notification {
	id: string;
	type: NotificationType;
	message: string;
	duration?: number;
}

class NotificationsState {
	notifications = $state<Notification[]>([]);

	add(type: NotificationType, message: string, duration = 5000) {
		const id = crypto.randomUUID();
		const notification: Notification = { id, type, message, duration };
		this.notifications = [...this.notifications, notification];

		if (duration > 0) {
			setTimeout(() => this.remove(id), duration);
		}

		return id;
	}

	remove(id: string) {
		this.notifications = this.notifications.filter((n) => n.id !== id);
	}

	success(message: string, duration?: number) {
		return this.add('success', message, duration);
	}

	error(message: string, duration?: number) {
		return this.add('error', message, duration);
	}

	info(message: string, duration?: number) {
		return this.add('info', message, duration);
	}

	warning(message: string, duration?: number) {
		return this.add('warning', message, duration);
	}

	clear() {
		this.notifications = [];
	}
}

export const notifications = new NotificationsState();
