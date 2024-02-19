#include<iostream>
#include<string>
#include<cmath>

struct Node {
	int num;
	Node* next;
	Node(int d) :num(d), next(nullptr) {}
};

void append(Node*& head, int num) {
	Node* newNode = new Node(num);
	if (!head) {
		head = newNode;
	}
	else {
		Node* temp = head;
		while (temp->next) {
			temp = temp->next;
		}
		temp->next = newNode;
	}
}

void reverse(Node*& head) {
	Node* prev = nullptr, * current = head, * next = nullptr;
	while (current != nullptr) {
		next = current->next;
		current->next = prev;
		prev = current;
		current = next;
	}
	head = prev;
}

Node* add(Node* num1, Node* num2) {
	Node* result = nullptr;
	Node* p = num1;
	Node* q = num2;
	int carry = 0;
	int tuiwei = 0;

	while (p != nullptr || q != nullptr) {
		int x = (p != nullptr) ? p->num : 0;
		int y = (q != nullptr) ? q->num : 0;
		int sum = x + y + carry - tuiwei;
		tuiwei = 0;
		while (sum < 0) {
			sum += 10;
			tuiwei++;
		}
		carry = sum / 10;
		append(result, sum % 10);

		if (p != nullptr) p = p->next;
		if (q != nullptr) q = q->next;
	}
	if (carry != 0)
		append(result, carry);
	return result;
}

Node* sub(Node* num1, Node* num2) {
	Node* result = nullptr;
	Node* p = num1;
	Node* q = num2;
	int carry = 0;
	int tuiwei = 0;
	while (p != nullptr || q != nullptr) {
		int x = (p != nullptr) ? p->num : 0;
		int y = (q != nullptr) ? q->num : 0;
		int sum = x - y + carry - tuiwei;
		tuiwei = 0;
		while (sum < 0) {
			sum += 10;
			tuiwei++;
		}
		carry = sum / 10;
		append(result, sum % 10);

		if (p != nullptr) p = p->next;
		if (q != nullptr) q = q->next;
	}
	if (carry != 0)
		append(result, carry);
	return result;
}

int length(Node* head) {
	//n为链表长度
	Node* temp = head;
	int n = 0;
	while (temp) {
		n++;
		temp = temp->next;
	}
	return n;
}

int delete0(Node*& head, int x) {
	while (x > 0) {
		if (head->num == 0 && head->next != nullptr) {
			Node* current = head;
			head = head->next;
			delete current;
			x--;
		}
		else
			return x;

	}return x;
}

void print(Node* head, int x) {
	int n = length(head);

	Node* temp = head;
	std::cout << temp->num;
	temp = temp->next;
	int i = 1;
	int j = (n - x) / 3;
	if (i == n - x && x != 0)std::cout << '.';
	while (temp) {

		if (j <= 0 || i % 3 != (n - x) % 3 || n - x <= 3)
			std::cout << abs(temp->num);
		else if (i < n - x) {
			std::cout << ',' << abs(temp->num);
			j--;
		}
		else std::cout << abs(temp->num);

		i++;
		temp = temp->next;
		if (i == n - x && x != 0)std::cout << '.';
	}
	std::cout << std::endl;
}

void clear(Node* head) {
	Node* temp = head;
	while (temp) {
		Node* current = temp;
		temp = temp->next;
		delete current;
	}
}

int xiaoshudian(std::string str) {
	int i = 1;
	for (char c : str) {
		if (c == '.') return int(str.length() - i);
		i++;
	}
	return 0;
}

int main() {
	int n;
	std::cin >> n;
	for (int i = 0; i < n; i++) {
		//输入所有数据
		char op;
		std::cin >> op;
		std::string str1, str2;
		std::cin >> str1;
		std::cin >> str2;

		//判断小数点位置
		int x1, x2;
		x1 = xiaoshudian(str1);
		x2 = xiaoshudian(str2);
		int x = std::max(x1, x2);
		while (x1 < x) {
			str1 += '0';
			x1++;
		}
		while (x2 < x) {
			str2 += '0';
			x2++;
		}

		//转换入链表
		Node* head1 = NULL;
		Node* head2 = NULL;
		if (str1[0] == '-') {
			for (int i = 1; i < str1.length(); i++) {
				if (str1[i] != ',' && str1[i] != '.') {
					append(head1, -(str1[i] - '0'));
				}
			}
		}
		else {
			for (char c : str1) {
				if (c != ',' && c != '.') {
					append(head1, (c - '0'));
				}
			}
		}
		if (str2[0] == '-') {
			for (int i = 1; i < str2.length(); i++) {
				if (str2[i] != ',' && str2[i] != '.') {
					append(head2, -(str2[i] - '0'));
				}
			}
		}
		else {
			for (char c : str2) {
				if (c != ',' && c != '.') {
					append(head2, (c - '0'));
				}
			}
		}

		reverse(head1);
		reverse(head2);

		Node* result = nullptr;
		if (op == '+')
			result = add(head1, head2);
		else
			result = sub(head1, head2);
		x = delete0(result, x);
		reverse(result);
		delete0(result, length(result) - x);
		print(result, x);
		clear(result);
		clear(head1);
		clear(head2);
	}
	return 0;
}