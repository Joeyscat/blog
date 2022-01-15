
db.article.updateMany(
  { comments: { $exists: true, $ne: [] } },
  {
    $set: {
      "comments.$[elem].author_name": "Joeyscat"
    }
  },
  {
    arrayFilters: [
      {
        "elem.author_id": ObjectId("61d7db265e0eb3f6ccfdc16d")
      }
    ]
  }
)

db.article.aggregate(
  {
    $match: { comments: { $exists: true, $ne: [] } }
  },
  {
    $project: {
      "comments": {
        "$reverseArray": "$comments"
      }
    }
  },
  {
    $merge: {
      into: "new_article_0112",
      on: "comments",
      whenMatched: "replace",
      whenNotMatched: "fail"
    }
  }
)

// 评论重新排序
db.article.find().forEach(function (doc) {
  var comments = doc.comments.reverse();
  db.article.updateOne({_id: doc._id}, { $set: { comments: comments } });
})
