
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