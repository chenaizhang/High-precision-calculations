#include<iostream>
#include<string>
#include<cmath>
#include <climits>

struct Node {
	int num;
	Node* next;
	Node(int d) :num(d), next(nullptr) {}
};

int compare(Node*& num1, Node*& num2);
bool ispositive(Node* head);
void negation(Node*& head);
bool neg_big(Node* head1, Node* head2);
int delete0(Node*& head, int x);
void append(Node*& head, int num);
void reverse(Node*& head);
bool toosmall(Node*& head);
Node* add(Node* num1, Node* num2);
Node* sub(Node* num1, Node* num2);
Node* subforchu(Node*& num1, Node*& num2);
void abs_(Node*& head);
bool dayu0(Node*& head);
Node* cheng(Node*& head1, Node*& head2);
int length(Node* head);
void print_1(Node* head, int x);
void print_2(Node* head, int x);
bool is0(Node* head);
Node* chu(Node*& head1, Node*& head2);
void clear(Node* head);
int xiaoshudian(std::string str);

int main() {
	//输入数据词数
	int n;
	std::cin >> n;

	for (int i = 0; i < n; i++) {
		//输入所有数据
		char op;
		std::cin >> op;
		std::string str1, str2;
		std::cin >> str1;
		std::cin >> str2;

		//判断为几位小数
		int x1, x2;
		x1 = xiaoshudian(str1);
		x2 = xiaoshudian(str2);
		int x;
		if (op == '+' || op == '-') {
			x = std::max(x1, x2);
			while (x1 < x) {
				str1 += '0';
				x1++;
			}
			while (x2 < x) {
				str2 += '0';
				x2++;
			}
		}
		
		//转换入链表
		Node* head1 = NULL;
		Node* head2 = NULL;
		if (str1[0] == '-') {
			for (size_t i = 1; i < str1.length(); i++) {
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
			for (size_t i = 1; i < str2.length(); i++) {
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

		//反转链表，开始模拟手算
		reverse(head1);
		reverse(head2);

		Node* result = nullptr;
		if (op == '*') {
			result = cheng(head1, head2);
			x = x1 + x2;
			x = delete0(result, x);
			reverse(result);
			delete0(result, length(result) - x - 1);
			print_2(result, x);
		}
		else if(op=='/'){
			if (toosmall(head2)) {
				std::cout << "ERROR" << std::endl;
			}
			else {
				x = 20 + x1 - x2;
				result = chu(head1, head2);
				x = delete0(result, x);
				reverse(result);
				delete0(result, length(result) - x - 1);
				print_2(result, x);
			}
		}
		else if(op == '+') {
			result = add(head1, head2);
			x = delete0(result, x);
			reverse(result);
			delete0(result, length(result) - x);
			print_1(result, x);
		}
		else if(op=='-'){
			result = sub(head1, head2);
			x = delete0(result, x);
			reverse(result);
			delete0(result, length(result) - x);
			print_1(result, x);
		}

		clear(result);
 		clear(head1);
		clear(head2);
	}
	return 0;
}

bool neg_big(Node* head1, Node* head2) {
	Node* pos, * neg;
	if (ispositive(head1) == 1) {
		pos = head1;
		neg = head2;
	}
	else {
		pos = head2;
		neg = head1;
	}
	negation(neg);
	if (compare(pos, neg) == -1) {
		negation(neg);
		return 1;
	}
	negation(neg);
	return 0;
}

bool ispositive(Node* head) {
	Node* current = head;
	while (current != nullptr) {
		if (current->num < 0)
			return 0;
		current = current->next;
	}
	return 1;
}

void negation(Node*& head) {
	Node* temp = head;
	while (temp) {
		temp->num = -temp->num;
		temp = temp->next;
	}
}
//删除反转后的链表头部的0，并返回小数点后位数
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

bool toosmall(Node*& head) {
	reverse(head);
	Node* temp = head;
	for (int i = 0; i < 7 && temp; i++) {
		if (temp->num != 0) {
			reverse(head);
			return 0;
		}
		temp = temp->next;
	}
	reverse(head);
	return 1;
}

Node* add(Node* num1, Node* num2) {
	Node* result = nullptr;
	Node* p = num1;
	Node* q = num2;
	int carry = 0;
	int tuiwei = 0;

	if (int((ispositive(num1)) + int(ispositive(num2)) == 1 && neg_big(num1, num2)) || int((ispositive(num1))) + int(ispositive(num2)) == 0) {
		negation(num1);
		negation(num2);
		result = add(num1, num2);
		negation(result);
		return result;
	}
	else {
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
}

Node* sub(Node* num1, Node* num2) {
	negation(num2);
	return add(num1, num2);
}

Node* subforchu(Node*& num1, Node*& num2) {
	Node* result = nullptr;
	Node* p = num1;
	Node* q = num2;
	int carry = 0;
	int tuiwei = 0;
	int sum;
	while (p != nullptr || q != nullptr) {
		int x = (p != nullptr) ? p->num : 0;
		int y = (q != nullptr) ? q->num : 0;
		sum = x - y + carry - tuiwei;
		tuiwei = 0;
		while (sum < 0) {
			sum += 10;
			tuiwei++;
		}
		carry = sum / 10;
		append(result, sum % 10);
		p->num = (sum % 10);


		if (p != nullptr) p = p->next;
		if (q != nullptr) q = q->next;
	}
	if (carry != 0) {
		append(result, carry);
		append(num1, carry);
	}
	return result;
}

void abs_(Node*& head) {
	Node* temp = head;
	while (temp) {
		temp->num = abs(temp->num);
		temp = temp->next;
	}
}

bool dayu0(Node*& head) {
	Node* temp = head;
	while (temp) {
		if (temp->num > 0) {
			return 1;
		}
		temp = temp->next;
	}
	return 0;
}

Node* cheng(Node*& head1, Node*& head2) {

	Node* result = nullptr;
	Node* temp1 = head1;
	int zhishu = 0;
	bool a = dayu0(head1);
	bool b = dayu0(head2);
	abs_(head1);
	abs_(head2);

	if (a - b == 0) {
		while (temp1) {
			Node* temp2 = head2;
			Node* tempresult = nullptr;
			int carry = 0;

			while (temp2) {
				int product = temp1->num * temp2->num + carry;
				carry = product / 10;
				append(tempresult, product % 10);
				temp2 = temp2->next;
			}
			if (carry != 0)
				append(tempresult, carry);
			for (int i = 0; i < zhishu; i++) {
				reverse(tempresult);
				append(tempresult, 0);
				reverse(tempresult);
			}

			if (result == nullptr) {
				result = tempresult;
			}
			else {
				result = add(result, tempresult);
			}
			zhishu++;
			temp1 = temp1->next;
		}
		return result;
	}
	else {
		while (temp1) {
			Node* temp2 = head2;
			Node* tempresult = nullptr;
			int carry = 0;

			while (temp2) {
				int product = temp1->num * temp2->num + carry;
				carry = product / 10;
				append(tempresult, product % 10);
				temp2 = temp2->next;
			}
			if (carry != 0)
				append(tempresult, carry);
			for (int i = 0; i < zhishu; i++) {
				reverse(tempresult);
				append(tempresult, 0);
				reverse(tempresult);
			}

			if (result == nullptr) {
				result = tempresult;
			}
			else {
				result = add(result, tempresult);
			}
			zhishu++;
			temp1 = temp1->next;
		}
		negation(result);
		return result;
	}
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

void print_1(Node* head, int x) {
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

void print_2(Node* head, int x) {
	int n = length(head);
	Node* temp = head;
	std::cout << temp->num;
	temp = temp->next;
	int i = 1;
	int j = (n - x) / 3;
	int c = INT_MAX;
	if (i == n - x && x != 0) {
		c = 9;
		std::cout << '.';
	}
	while (temp && c > 0) {
		c--;
		if (j <= 0 || i % 3 != (n - x) % 3 || n - x <= 3)
			std::cout << abs(temp->num);
		else if (i < n - x) {
			std::cout << ',' << abs(temp->num);
			j--;
		}
		else std::cout << abs(temp->num);

		i++;
		temp = temp->next;
		if (i == n - x && x != 0) {
			std::cout << '.';
			c = 9;
		}
	}
	if (c == 0) {
		if (!temp->next)
			std::cout << abs(temp->num);
		else if (abs(temp->next->num) < 5)
			std::cout << abs(temp->num);
		else
			std::cout << 1 + abs(temp->num);
	}

	std::cout << std::endl;
}

int compare(Node*& num1, Node*& num2) {
	reverse(num1);
	reverse(num2);
	while (num1->num == 0 && num1->next) {
		num1 = num1->next;
	}
	while (num2->num == 0 && num2->next) {
		num2 = num2->next;
	}
	int len1 = length(num1);
	int len2 = length(num2);

	// 如果数字长度不同，返回长度更长的数值为较大值
	if (len1 > len2) {
		reverse(num1);
		reverse(num2);
		return 1;
	}
	if (len1 < len2) {
		reverse(num1);
		reverse(num2);
		return -1;
	}
	// 长度相同时，逐位比较数值
	Node* temp1 = num1;
	Node* temp2 = num2;
	while (temp1 && temp2) {
		if (temp1->num > temp2->num) {
			reverse(num1);
			reverse(num2);
			return 1;
		}
		if (temp1->num < temp2->num) {
			reverse(num1);
			reverse(num2);
			return -1;
		}
		temp1 = temp1->next;
		temp2 = temp2->next;
	}
	reverse(num1);
	reverse(num2);
	return 0;  // 数值相等
}

bool is0(Node* head) {
	Node* temp = head;

	while (temp) {
		if (temp->num != 0)return 0;
		temp = temp->next;
	}
	return 1;
}

Node* chu(Node*& head1, Node*& head2) {
	Node* result = nullptr;

	bool a = dayu0(head1);
	bool b = dayu0(head2);
	reverse(head1);
	if (a - b == 0) {
		abs_(head1);
		abs_(head2);
		Node* next = head1;
		Node* current = head1;
		while (current) {
			next = current->next;
			current->next = nullptr;
			int n = 0;
			reverse(head1);
			while (compare(head1, head2) != -1) {
				subforchu(head1, head2);
				n++;
			}
			reverse(head1);
			append(result, n);
			current->next = next;
			current = current->next;
		}
		reverse(result);

		if (is0(head1)) {
			reverse(result);
			for (int b = 0; b < 20; b++) {
				append(result, 0);
			}
			reverse(result);
			return result;
		}

		for (int i = 0; i < 20; i++) {
			append(head1, 0);
			int n = 0;
			reverse(head1);
			while (compare(head1, head2) != -1) {
				subforchu(head1, head2);
				n++;
			}
			reverse(head1);
			reverse(result);
			append(result, n);
			reverse(result);
			if (is0(head1)) {
				reverse(result);
				for (int b = 0; b < 19 - i; b++) {
					append(result, 0);
				}
				reverse(result);
				return result;
			}
		}
		return result;
	}
	else {
		abs_(head1);
		abs_(head2);
		Node* next = head1;
		Node* current = head1;
		while (current) {
			next = current->next;
			current->next = nullptr;
			int n = 0;
			reverse(head1);
			while (compare(head1, head2) != -1) {
				subforchu(head1, head2);
				n++;
			}
			reverse(head1);
			append(result, n);
			current->next = next;
			current = current->next;
		}
		reverse(result);

		if (is0(head1)) {
			reverse(result);
			for (int b = 0; b < 20; b++) {
				append(result, 0);
			}
			reverse(result);
			negation(result);
			return result;
		}

		for (int i = 0; i < 20; i++) {
			append(head1, 0);
			int n = 0;
			reverse(head1);
			while (compare(head1, head2) != -1) {
				subforchu(head1, head2);
				n++;
			}
			reverse(head1);
			reverse(result);
			append(result, n);
			reverse(result);
			if (is0(head1)) {
				reverse(result);
				for (int b = 0; b < 19 - i; b++) {
					append(result, 0);
				}
				reverse(result);
				negation(result);
				return result;
			}
		}
		negation(result);
		return result;
	}
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
		if (c == '.')
			return int(str.length() - i);
		i++;
	}
	return 0;
}
