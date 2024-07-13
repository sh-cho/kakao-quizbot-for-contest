## kakao-quizbot

<p>
  <img src="https://github.com/user-attachments/assets/9ff5c0d1-413e-467e-98a0-70f298af5f4b" alt="Text quiz example" width="300"/>
  <img src="https://github.com/user-attachments/assets/379ba849-24bf-444f-8ea7-d3b9187f77f8" alt="Flag quiz example" width="300"/>
</p>

### Instructions
```sh
rustup target add x86_64-unknown-linux-gnu
brew tap SergioBenitez/osxct

cargo build --release --target x86_64-unknown-linux-gnu
scp target/x86_64-unknown-linux-gnu/release/kakao-quizbot ~~~
```
Build for x86 on m1 mac

### TODO

- [ ] x86 배포 좀 쉽게..
- [ ] 문제 Timeout (30초?) with Event API
  - 답 알려줘야됨
- [ ] Refactoring
- [ ] Redis connection pool 연결
- [ ] 멀티 정답 (ex. 국기 문제에서 `["미국", "미합중국", ...]`)
- [ ] 난이도 조절?
- [x] 다양한 말풍선: SimpleImage
- [x] 카테고리 선택해서 시작하기

아직 생각중인 부분..

- 명시적 정답 커맨드 없이 바로 답 입력받기
  - 커맨드랑 겹치지 않도록 조심하는건 귀찮지 않을까?
  - 하지만 매번 정답 커맨드를 입력하는건 좀 번거롭다
  - [ ] TODO: 겜중이면 바로 입력, 아니라면 도움말 출력
