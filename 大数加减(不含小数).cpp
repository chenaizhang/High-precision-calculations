#include<iostream>
#include<string>
#include<cmath>

struct Node {
	int num;
	Node* next;
	Node(int d) :num(d), next(nullptr){}
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
		int sum = x + y + carry-tuiwei;
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
		int sum = x - y + carry-tuiwei;
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

void print(Node* head) {
	Node* temp1 = head;
	int n = 0;
	while (temp1) {
		n++;
		temp1 = temp1->next;
	}
	Node* temp = head;
	std::cout << temp->num;
	temp = temp->next;
	int i = 1;
	int j = n / 3;
	while (temp) {

		if (j <= 0 || i % 3 != n % 3 || n <= 3)
			std::cout << abs(temp->num);
		else{
			std::cout << ',' << abs(temp->num);
		j--;
	}
		i++;
		temp = temp->next;
		
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
		
		//转换入链表
		Node* head1 = NULL;
		Node* head2 = NULL;
		if (str1[0] == '-') {
			for (int i = 1; i < str1.length(); i++) {
				if (str1[i] != ',') {
					append(head1, -(str1[i] - '0'));
				}
			}
		}
		else {
			for (char c : str1) {
				if (c != ',' ) {
					append(head1, (c - '0'));
				}
			}
		}
		if (str2[0] == '-') {
			for (int i = 1; i < str2.length(); i++) {
				if (str2[i] != ',') {
					append(head2, -(str2[i] - '0'));
				}
			}
		}
		else {
			for (char c : str2) {
				if (c != ',') {
					append(head2, (c - '0'));
				}
			}
		}

		reverse(head1);
		reverse(head2);

		Node* result = nullptr;
		head1 = nullptr;
		if (op == '+')
			result = add(head1, head2);
		else
			result = sub(head1, head2);

		reverse(result);
		print(result);
		clear(result);

	}
	return 0;
}