[Rust](https://www.rust-lang.org/) + [Poem](https://github.com/poem-web/poem/) + [MongoDB](https://www.mongodb.com/) 实现的博客。

界面风格完全照搬 - Rust语言中文社区（[https://rustcc.cn/](https://rustcc.cn/)），当然功能上还要更 ~~简陋~~ 简单；

### API

- 首页/文章列表
`GET` /

- 登录页面
`GET` /signin

- Gitee登录
`GET` /gitee/signin

- 账户界面
`GET` /account

- 退出登录
`get` /signout

- 发布文章页面
`GET` /article/publish

- 发布文章
`POST` /article/publish

- 修改文章页面
`GET` /article/edit

- 修改文章
`POST` /article/edit

- 文章详情
`GET` /article

- 发表评论
`GET` /comment/new

